// PURE-RUST PIPELINE: Stable Camera Setup, Inference, and Coordinate Parsing

use v4l::{
    buffer::Type,
    capability::Flags,
    device::{self, Device},
    io::{mmap::Stream, traits::CaptureStream},
    video::Capture,
    FourCC,
    Format,
};

use image::{ImageBuffer, Rgb, imageops::FilterType}; 
use std::io;
use std::time::Instant;

// FIX: Import Array and IxDyn from tract_core to ensure version compatibility
use tract_core::ndarray::{Array, IxDyn};

use tract_core::model::{Graph, TypedFact};
use tract_core::internal::SimplePlan; 
use tract_core::ops::{TypedOp}; 
use tract_onnx::prelude::{tvec, Framework, InferenceModelExt, Tensor}; 
use tract_core::runtime::Runnable; 
use tract_onnx::prelude::TVec; 

// --- CONFIGURATION ---
const MODEL_WIDTH: u32 = 32; 
const MODEL_HEIGHT: u32 = 32; 
const MODEL_BATCH: usize = 10; 

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const MODEL_PATH: &str = "./vtuber_tracker/model.onnx"; 

struct Camera {
    device: Device,
    stream: Option<Stream<'static>>,
    format: Format,
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>, 
}

impl Camera {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("--- CAMERA: INITIALIZING ---");
        let device = Camera::get_camera_device()?;
        
        let format = Format::new(WIDTH, HEIGHT, FourCC::new(b"YUYV"));
        let current_format = Capture::set_format(&device, &format)?;

        let mut stream = Stream::with_buffers(&device, Type::VideoCapture, 4)?;
        v4l::io::traits::Stream::start(&mut stream)?;
        println!("INFO: Stream started successfully.");

        println!("INFO: Loading ONNX model from: {}", MODEL_PATH);
        
        let model = tract_onnx::onnx() 
            .model_for_path(MODEL_PATH)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Tract read error: {}", e)))?
            .into_optimized()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Optimization failed: {}", e)))?
            .into_runnable() 
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Runnable conversion failed: {}", e)))?;

        println!("INFO: Model loaded.");

        Ok(Camera {
            device,
            stream: Some(unsafe { std::mem::transmute(stream) }),
            format: current_format,
            model, 
        })
    }
    
    fn get_camera_device() -> Result<Device, io::Error> {
        for i in 0..10 {
            let path = format!("/dev/video{}", i);
            if let Ok(device) = device::Device::with_path(path) {
                let caps = device.query_caps()?;
                if caps.capabilities.contains(Flags::VIDEO_CAPTURE) {
                    println!("INFO: Found suitable camera at: /dev/video{}", i);
                    return Ok(device);
                }
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "No suitable video capture device found"))
    }
    
    fn capture_frame(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let stream = self.stream.as_mut().expect("Camera stream is missing!");
        let (raw_data, _) = stream.next().map_err(|e| format!("Stream read error: {}", e))?;
        Ok(raw_data.to_vec())
    }
}

fn convert_yuyv_to_rgb(yuyv_data: &[u8]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut rgb_pixels = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for chunk in yuyv_data.chunks_exact(4) { 
        let y0 = chunk[0] as f32;
        let u0 = chunk[1] as f32 - 128.0;
        let y1 = chunk[2] as f32;
        let v0 = chunk[3] as f32 - 128.0;
        
        let r0 = y0 + 1.402 * v0;
        let g0 = y0 - 0.344 * u0 - 0.714 * v0;
        let b0 = y0 + 1.772 * u0;
        let r1 = y1 + 1.402 * v0;
        let g1 = y1 - 0.344 * u0 - 0.714 * v0;
        let b1 = y1 + 1.772 * u0;

        rgb_pixels.push(r0.clamp(0.0, 255.0) as u8);
        rgb_pixels.push(g0.clamp(0.0, 255.0) as u8);
        rgb_pixels.push(b0.clamp(0.0, 255.0) as u8);
        rgb_pixels.push(r1.clamp(0.0, 255.0) as u8);
        rgb_pixels.push(g1.clamp(0.0, 255.0) as u8);
        rgb_pixels.push(b1.clamp(0.0, 255.0) as u8);
    }
    return ImageBuffer::from_vec(WIDTH, HEIGHT, rgb_pixels).expect("Failed to create RGB image buffer");
}

fn image_to_tensor(image: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Array<f32, IxDyn> {
    let resized_image = image::imageops::resize(&image, MODEL_WIDTH, MODEL_HEIGHT, FilterType::Triangle);
    let (width, height) = resized_image.dimensions();
    let data = resized_image.into_vec(); 
    
    let array = Array::from_shape_vec((height as usize, width as usize, 3), data)
        .expect("Failed to create ndarray from image data");

    let array_f32 = array.mapv(|x| (x as f32) / 255.0); 
    let chw_array = array_f32.permuted_axes([2, 0, 1]);

    let batch_array = Array::from_shape_fn((MODEL_BATCH, 3, height as usize, width as usize), |(_, c, h, w)| {
        chw_array[[c, h, w]]
    });

    batch_array.into_dyn()
}

fn array_to_tract_tensor(array: Array<f32, IxDyn>) -> Tensor {
    let shape = array.shape().to_vec();
    #[allow(deprecated)] 
    let data = array.into_raw_vec(); 
    let data_tvec: TVec<f32> = data.into();
    tract_core::internal::Tensor::from_shape(&shape, &data_tvec).expect("Failed to construct Tract Tensor")
}

fn find_face_coordinates(output_tensor: &Array<f32, IxDyn>) -> Option<(u32, u32)> {
    let shape = output_tensor.shape();
    if shape.len() < 4 { return None; }
    let grid_h = shape[2]; // 8
    let grid_w = shape[3]; // 8

    let mut max_val = -1.0;
    let mut max_x = 0;
    let mut max_y = 0;

    for y in 0..grid_h {
        for x in 0..grid_w {
            let val = output_tensor[[0, 0, y, x]];
            if val > max_val {
                max_val = val;
                max_x = x;
                max_y = y;
            }
        }
    }

    if max_val < 0.5 { 
        return None;
    }

    let screen_x = ((max_x as f32 + 0.5) / grid_w as f32) * WIDTH as f32;
    let screen_y = ((max_y as f32 + 0.5) / grid_h as f32) * HEIGHT as f32;

    Some((screen_x as u32, screen_y as u32))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut camera = Camera::new()?;
    let mut frame_count = 0;
    
    println!("\n--- STARTING MOTION TRACKING LOOP (Press Ctrl+C to exit) ---\n");
    let start_time = Instant::now();
    
    loop {
        let capture_start = Instant::now();
        let raw_frame_data = camera.capture_frame()?;
        frame_count += 1;
        
        let rgb_image = convert_yuyv_to_rgb(&raw_frame_data);
        let nd_array = image_to_tensor(rgb_image); 
        let input_tensor = array_to_tract_tensor(nd_array); 
        
        let inference_start = Instant::now(); // RESTORED TIMER
        
        // Run Inference
        let result = camera.model.run(tvec!(input_tensor.into())).map_err(|e| format!("Inference failed: {}", e))?;
        
        let inference_duration = inference_start.elapsed();
        let output_tensor = result[0].to_array_view::<f32>().map_err(|e| format!("Output tensor view failed: {}", e))?;
        
        // Interpret the "Mind" of the AI
        let face_coords = find_face_coordinates(&output_tensor.to_owned());
        
        if frame_count % 5 == 0 { 
            let elapsed_sec = start_time.elapsed().as_secs_f32();
            let fps = frame_count as f32 / elapsed_sec;
            
            let loc_str = match face_coords {
                Some((x, y)) => format!("FACE DETECTED at X:{:03} Y:{:03}", x, y),
                None => "NO FACE FOUND".to_string()
            };

            println!(
                "[{:05} Frames] FPS:{:.1} | {}",
                frame_count, fps, loc_str
            );
        }
    }
}
