# ASCII VTUBER PROJECT

This project implements a real-time 3D facial tracking and rendering system in Rust. The output is a terminal-based 3D avatar rendered using custom ASCII rasterization.

The project is structured as a Cargo Workspace to enforce modularity and separate the computer vision pipeline from the rendering pipeline.

---

## ARCHITECTURE

### 1. ascii_renderer (Output Module)

* Role: Handles 3D mathematics, asset loading, projection, and rasterizes the final scene into ASCII characters for terminal display.
* Input: Receives clean, numerical pose data (pitch, yaw, roll, blend shapes) from the vtuber_tracker module.
* Dependencies: glam, crossterm, gltf.

### 2. vtuber_tracker (Input Module)

* Role: Handles all video capture and computer vision processing, including real-time facial landmark detection and pose estimation.
* Output: Sends clean, numerical pose data to the ascii_renderer for visualization.
* Dependencies: (To be added based on target OS: Windows/Linux)