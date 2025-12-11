use glam::{Mat4, Vec3};

//Define the Renderer struct
pub struct Renderer {
    //3D transformation matrices
    view_matrix: Mat4,
    projection_matrix: Mat4,
    //Size of the terminal window
    width: u16,
    height: u16,
}

impl Renderer {
    pub fn new(width: u16, height: u16) -> Self {
        //Aspect ratio of the terminal
        let aspect_ratio = width as f32 / height as f32;

        // 1. Perspective Projection Matrix
        // Using 90 degrees (pi/2) FOV for a wide view
        let projection_matrix = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_2,
            aspect_ratio,
            0.1,
            100.0,
        );

        // 2. View Matrix
        // Camera positioned at (0,0,5) looking towards the origin
        let view_matrix = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0), // Eye position
            Vec3::new(0.0, 0.0, 0.0), // Target (where camera is looking)
            Vec3::Y,               // Up direction (0, 1, 0)
        );

        Renderer {
            view_matrix,
            projection_matrix,
            width,
            height
        }
            
    }


// Function to project a single 3D position onto the 2D screen
pub fn project(&self, position: Vec3) -> Vec3 {
    // 1. Model Matrix (Trivial identity matrix for now)
    let model_matrix = Mat4::IDENTITY;

    // 2. Combine the matrices into the mVP matrix (Model-View-Projection)
    let mvp = self.projection_matrix * self.view_matrix * model_matrix;

    // 3. Transform the 3D point into clip space (Homogenous Coordinates)
    let clip_pos = mvp.mul_vec4(position.extend(1.0));

    // 4. Perform Perspective Division (conversion to Normalized Device Coordinates)
    let ndc_pos = Vec3::new(
        clip_pos.x / clip_pos.w,
        clip_pos.y / clip_pos.w,
        clip_pos.z / clip_pos.w,
    );

    // 5. Convert NDC [-1, 1] to screen coordinates [0, width/height]
    let screen_x = (ndc_pos.x + 1.0) / 2.0 * self.width as f32;
    let screen_y = (1.0 - ndc_pos.y) / 2.0 * self.height as f32; // Invert Y for screen space

    // Return the screen position and the depth value
    Vec3::new(screen_x, screen_y, ndc_pos.z)
}
}