use glam::Vec3;
//3D model representation
#[derive(Debug, Clone)]
pub struct Vertex {
        pub position: Vec3, // 3D coordinates
        pub normal: Vec3,   // Normal vector
}
// Model structure containing vertices from GLTF file
#[derive(Debug)]
pub(crate) struct Model {
        pub vertices: Vec<Vertex>, // List of vertices
}

// Function to load a model from a GLTF file
pub(crate) fn load_model(path: &str) -> Result<Model, Box<dyn std::error::Error>> {
        let mut vertices = Vec::new();

        //  1. Import the GLTF file
        let (document, buffers, _) = gltf::import(path)?;

        // 2. Iterate through meshes in the GLTF document
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                // Access the reader for the primitive
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // 3. Read positions and normals
                if let (Some(positions), Some(normals)) = (reader.read_positions(), reader.read_normals()) {
                    for (position, normal) in positions.zip(normals) {
                        vertices.push(Vertex {
                            position: glam::Vec3::from(position),
                            normal: glam::Vec3::from(normal),
                        });
                    }
                } else {
                    return Err(Box::from("Primitive is missing POSITION or NORMAL attributes.".to_string()));
                }
            }
        }
        if vertices.is_empty() {
            return Err(Box::from("No vertices found in the model.".to_string()));
        }
        Ok(Model { vertices })
}