use serde::{Serialize, Deserialize};

// This struct is used by both the vtuber_tracker (to send) 
// and the ascii_renderer (to receive) the rotation data via UDP.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MotionData {
    pub pitch: f32, // Rotation around the X-axis (up/down head tilt)
    pub yaw: f32,   // Rotation around the Y-axis (left/right head turn)
    pub roll: f32,  // Rotation around the Z-axis (left/right shoulder tilt)
}

// UDP Port for communication between the two processes
pub const UDP_PORT: u16 = 4000;

// The loopback address for local communication
pub const TRACKER_ADDR: &str = "127.0.0.1";
