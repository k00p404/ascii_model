// main.rs
mod data;
mod opengl_render; 
mod terminal; 

use std::error::Error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use nalgebra::{Matrix4, Rotation3, Vector3};
use glutin::context::PossiblyCurrentContext;
use glutin::surface::Surface;
use glutin::prelude::GlSurface; // GlSurface Trait imported for the resize method!


fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new(); 
    
    let window_builder = WindowBuilder::new()
        .with_title("3D ASCII Vtuber Debug Window")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));

    // 2. Create the OpenGL Context and Surface
    let (window, surface, context) = opengl_render::init_opengl(window_builder, &event_loop) 
        .expect("Failed to initialize OpenGL context and surface.");
    
    // 3. Load the Model 
    let model = data::load_model("assets/Box.glb")?;
    println!("INFO: 3D Model loaded successfully: {}", model.name);
    
    // --- Temporary MVP Setup Variables ---
    let mut rotation_y: f32 = 0.0;
    
    // 4. Main Event Loop
    event_loop.run(move |event, window_target, control_flow| { 
        
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit; 
                }
                WindowEvent::Resized(size) => {
                    surface.resize(&context, size.width.try_into().unwrap(), size.height.try_into().unwrap());
                }
                _ => (),
            },
            
            Event::RedrawRequested(_) => {
                rotation_y += 0.01; 
                
                // --- MVP Matrix Calculation ---
                let rotation_matrix = Rotation3::from_axis_angle(&Vector3::y_axis(), rotation_y);
                let translation_matrix = Matrix4::new_translation(&Vector3::new(0.0, 0.0, -2.0));
                
                let model_matrix = translation_matrix * rotation_matrix.to_homogeneous();
                
                // 5. OpenGL Rendering and Data Processing
                opengl_render::render_frame(&context, &surface, model_matrix); 
                window.request_redraw(); 
            }
            
            _ => (),
        }
    }); // <-- FIX: Removed the question mark here.

    Ok(())
}
