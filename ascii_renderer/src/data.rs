// data.rs
use std::error::Error;

// A simple placeholder struct for the loaded 3D data
#[allow(dead_code)] 
pub struct Model {
    pub name: String,
    pub vertices: Vec<(f32, f32, f32)>, // (x, y, z)
    pub indices: Vec<(usize, usize, usize)>, // Triangles
}

pub fn load_model(path: &str) -> Result<Model, Box<dyn Error>> {
    // Note: This function will eventually load a GLB file using the gltf crate.
    // For now, we mock the data for a centered test cube's vertices.
    
    // Check if path is valid, even though we use mock data
    if path.is_empty() {
        return Err("Model path cannot be empty.".into());
    }

    // Mock a simple centered cube's vertices (four points for now)
    let vertices = vec![
        (0.0, 0.0, 0.0), // Centered test point
        (-1.0, 1.0, 0.0),
        (-1.0, -1.0, 0.0),
        (1.0, -1.0, 0.0),
    ];
    let indices = vec![
        (0, 1, 2), 
        (0, 2, 3), 
    ];
    
    Ok(Model {
        name: path.to_string(),
        vertices,
        indices,
    })
}
