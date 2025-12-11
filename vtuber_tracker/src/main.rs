use v4l::{
    buffer::Type,
    capability::Flags,
    device::{self, Device},
    io::mmap::Stream,
    video::{capture::Parameters, format::Format},
};
use std::{thread, time::Duration};

// Function to find a suitable webcam device
pub fn get_camera_device() -> Result<Device, std::io::Error> {
    // We will iterate through standard device paths (/dev/video0, /dev/video1, etc.)
    // Note: On Windows, this may require specific driver installation or a different crate
    // if v4l does not manage the camera via a Linux-compatibility layer.
    for i in 0..10 {
        let path = format!("/dev/video{}", i);
        if let Ok(device) = device::Device::with_path(path) {
            let caps = device.query_caps()?;
            
            // Check if the device is a video capture device
            if caps.capabilities.contains(Flags::VIDEO_CAPTURE) {
                println!("INFO: Found suitable camera at: /dev/video{}", i);
                return Ok(device);
            }
        }
    }
    
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No suitable video capture device found (tried /dev/video0 to /dev/video9)",
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- VTUBER TRACKER: INITIALIZING CAMERA ---");

    // 1. Find and Open the Camera
    let dev = get_camera_device()?;
    
    // 2. Query and Print Camera Information
    let caps = dev.query_caps()?;
    println!("Device Name: {}", caps.card);

    // 3. Set Desired Video Format (Common default: YUYV at 640x480)
    let format = Format::new(640, 480, b"YUYV")?;
    
    let current_format = dev.set_format(&format)?;
    println!(
        "Set Format: {}x{} ({})",
        current_format.width,
        current_format.height,
        current_format.fourcc.to_string().trim(),
    );

    // 4. Placeholder for Image Processing Loop
    println!("\nCamera setup successful. Ready for frame capture...");
    
    // Keep the thread alive for inspection
    thread::sleep(Duration::from_secs(3)); 

    Ok(())
}