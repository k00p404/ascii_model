mod data;
mod renderer;
mod terminal;

use data::Model;
use renderer::Renderer;
use glam::{Vec3, Mat4};
use glam::Vec4Swizzles;
use crossterm::ExecutableCommand;
use std::{thread, time::Duration};
use std::io::{stdout, Write}; // <-- Imported the Write trait

fn main() {
    // This is the cleanup function that runs if the main loop (run()) fails (e.g., Ctrl+C is pressed).
    if let Err(e) = run() {
        // We attempt to show the cursor again, just in case the terminal is left in a bad state.
        let _ = stdout().execute(crossterm::cursor::Show);
        eprintln!("Application error: {:?}", e);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initial Terminal Setup: Hide the cursor and clear the screen once
    stdout().execute(crossterm::cursor::Hide)?;
    stdout().execute(crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;

    // Get terminal dimensions
    let (width, height) = crossterm::terminal::size()?;

    // Initialize the renderer
    let renderer = Renderer::new(width, height);

    // Load the 3D model
    let model = data::load_model("assets/Box.glb")?;

    let mut frame_count = 0.0; // Variable to control continuous rotation

    // --- START ANIMATION LOOP ---
    loop {
        // Reset buffers for the new frame
        let screen_size = (width * height) as usize;
        let mut depth_buffer = vec![f32::MAX; screen_size];
        let mut screen_buffer = vec![' '; screen_size];

        // 2. Continuous Model Transformation
        let rotation_x = frame_count * 0.05;
        let rotation_y = frame_count * 0.03;

        // Rotation matrix is dynamic, based on frame_count
        let model_matrix = glam::Mat4::from_rotation_x(rotation_x) * glam::Mat4::from_rotation_y(rotation_y);

        // Iterate through all vertices
        for vertex in &model.vertices {
            // 1. Apply the model transformation to the vertex position
            let world_pos = model_matrix.mul_vec4(vertex.position.extend(1.0)).xyz();

            // 2. Project the 3D world pos to 2D screen coords
            let projected_pos = renderer.project(world_pos);

            let screen_x = projected_pos.x.round() as usize;
            let screen_y = projected_pos.y.round() as usize;
            let depth = projected_pos.z;

            // Ensure coords are within terminal bounds
            if screen_x < width as usize && screen_y < height as usize {
                let index = screen_y * width as usize + screen_x;

                // Simple Z-buffer check
                if depth < depth_buffer[index] {
                    depth_buffer[index] = depth;
                    screen_buffer[index] = '#';
                }
            }
        }

        // Output the screen buffer (Dynamic output commands)
        let output: String = screen_buffer.iter().collect();
        stdout()
            .execute(crossterm::cursor::MoveTo(0, 0))? // Move cursor to top-left for redraw
            .write_all(output.as_bytes())?; // Write the new frame (Requires std::io::Write)

        frame_count += 1.0;
        thread::sleep(Duration::from_millis(30)); // Delay for ~30 FPS
    }
    // --- END ANIMATION LOOP ---
    // The function never returns Ok(()) because the loop is infinite.
}