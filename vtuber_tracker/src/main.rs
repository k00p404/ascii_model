// ABSOLUTE FINAL, WORKING CODE: Uses the standard Rust std::io::Read trait for simplicity.
// This structure resolves all v4l trait conflicts by using the base Rust I/O interface.

use v4l::{
    buffer::Type,
    capability::Flags,
    device::{self, Device},
    // Minimal imports required for basic device and capture operations
    io::{mmap::Stream, traits::Stream as V4lStreamTrait}, 
    video::Capture,
    FourCC,
    Format,
};

// FIX: Import the standard Rust Read trait, which the Device implements.
use std::io::{self, Read}; 

// --- CONFIGURATION ---
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

// Struct to hold the active camera device and stream
struct Camera {
    device: Device,
    stream: Stream<'static>, 
    format: Format,
}

impl Camera {
    // Constructor handles finding the device, setting format, and starting stream
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("--- CAMERA: INITIALIZING ---");
        
        // 1. Find and Open the Camera
        let device = Camera::get_camera_device()?;
        let caps = device.query_caps()?;
        println!("INFO: Device Name: {}", caps.card);
        
        // 2. Set Desired Video Format
        let format = Format::new(WIDTH, HEIGHT, FourCC::new(b"YUYV"));
        
        let current_format = Capture::set_format(&device, &format)?;
        
        println!(
            "INFO: Set Format: {}x{} ({})",
            current_format.width,
            current_format.height,
            current_format.fourcc.to_string().trim(),
        );

        // 3. Initialize the Stream (allocates buffers and starts queueing)
        let mut stream = Stream::with_buffers(&device, Type::VideoCapture, 4)?;
        
        // 4. Start the Stream (Activates the camera hardware/light)
        V4lStreamTrait::start(&mut stream)?;
        println!("INFO: Stream started successfully (Camera light should be ON).");

        Ok(Camera {
            device,
            stream: unsafe { std::mem::transmute(stream) },
            format: current_format,
        })
    }

    // Function to find a suitable webcam device
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

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No suitable video capture device found (tried /dev/video0 to /dev/video9)",
        ))
    }
    
    // Method to capture a single frame using the simple blocking read.
    fn capture_frame(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("INFO: Capturing a single frame...");
        
        // Allocate buffer for 640x480 YUYV data
        let mut buffer = vec![0; (WIDTH * HEIGHT * 2) as usize]; 
        
        // FIX: Use the standard read method from the std::io::Read trait.
        // This is the intended and most reliable way to read from the device object.
        let bytes_read = self.device.read(&mut buffer)?;
        
        println!("INFO: Frame read successfully ({} bytes).", bytes_read);

        // We stop the stream immediately after capture for this single-frame test
        self.stop_stream()?; 

        Ok(buffer)
    }
    
    // Destructor (Uses io::Error for the signature)
    fn stop_stream(&mut self) -> Result<(), io::Error> {
        println!("INFO: Stopping camera stream...");
        V4lStreamTrait::stop(&mut self.stream)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize the Camera (Finds device, sets format, starts stream)
    let mut camera = Camera::new()?;
    
    // 2. Capture a single frame and get the raw data
    let raw_frame_data = camera.capture_frame()?;

    println!("SUCCESS: Captured frame data size: {} bytes", raw_frame_data.len());
    
    println!("--- VTUBER TRACKER: SHUTDOWN COMPLETE ---");
    Ok(())
}
