use glow::HasContext;
use me_learning_opengl::{RenderHandler, SliceAsBytes};
use std::{path::Path, time::Instant};

const VERTEX_SHADER_SRC: &str = include_str!("textures_01/vertex.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("textures_01/fragment.glsl");

// Make a square
const TRI_VERTICES: &[f32] = &[
    // Positions (3)       // Colors (4)   // TexCoords (2)
    -0.5, -0.5, 0.0, 1., 0., 0., 1., 0., 0., // bottom left
    0.5, -0.5, 0.0, 0., 1., 0., 1., 0., 1., // bottom right
    0.5, 0.5, 0.0, 0., 0., 1., 1., 1., 1., // top right
    -0.5, 0.5, 0.0, 0.5, 0.5, 0.5, 1., 1., 0., // top left
];
const TRI_VERTICE_INDEXES: &[u32] = &[
    0, 1, 2, // First triangle
    0, 2, 3, // Second triangle
];

struct Textures01 {
    /// A compiled and linked shader program: Combines the vertex shader and the
    /// fragment shader into a usable shader program.
    shader_program: u32,
    /// Vertex Array Object: It's like a vertex attributes configuration
    /// "preset"
    vao: u32,
    texture0: u32,
    texture1: u32,
    /// The shader program uniform for the time the program has been running
    time_uniform: u32,
    /// The instant that the renderer was initialized
    start_time: Instant,
}

impl RenderHandler for Textures01 {
    fn init(gl: &mut glow::Context) -> Self {
        unsafe {
            //
            // Create and link shaders
            //

            // Create a vertex shader
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            // Load the shader's GLSL source
            gl.shader_source(vertex_shader, VERTEX_SHADER_SRC);
            // Compile the vertex shader
            gl.compile_shader(vertex_shader);
            // Check for shader compile errors
            handle_shader_compile_errors(gl, vertex_shader);

            // Create a fragment shader
            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            // Load the shader's GLSL source
            gl.shader_source(fragment_shader, FRAGMENT_SHADER_SRC);
            // Compile the fragment shader
            gl.compile_shader(fragment_shader);
            handle_shader_compile_errors(gl, fragment_shader);

            // Create a shader program to link our shaders to
            let shader_program = gl.create_program().unwrap();
            // Add both shaders to the program
            gl.attach_shader(shader_program, vertex_shader);
            gl.attach_shader(shader_program, fragment_shader);
            // Link the program
            gl.link_program(shader_program);
            // Handle link errors
            handle_program_link_errors(gl, shader_program);

            // Get the index for the time uniform from our shader program
            let time_uniform = gl.get_uniform_location(shader_program, "time").unwrap();
            // Use the shader program
            gl.use_program(Some(shader_program));
            // Set the the face color uniform value ( start at zero )
            gl.uniform_1_f32(Some(&time_uniform), 0.);

            // Delete our shader objects. Now that they are linked we don't need them.
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            //
            // Create vertext array and vertex buffer
            //

            // Create vertex array object that will store our vertex attribute
            // config like a "preset".
            let vao = gl.create_vertex_array().unwrap();

            // Create vertex buffer object that stores the actuall vertex data
            let vbo = gl.create_buffer().unwrap();

            // Bind the VAO so that all vertex attribute operations will be
            // recorded in that VAO
            gl.bind_vertex_array(Some(vao));

            // Bind the vbo as the ARRAY_BUFFER
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            // Upload vertex data to the VBO
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                TRI_VERTICES.as_mem_bytes(),
                glow::STATIC_DRAW,
            );

            // Create the element buffer object ( EBO ) for indexing into the vertices in the VBO
            let ebo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                TRI_VERTICE_INDEXES.as_mem_bytes(),
                glow::STATIC_DRAW,
            );

            // Describe our vertex position attribute data format
            gl.vertex_attrib_pointer_f32(
                // Corresponds to `location = 0` in the vertex shader
                0,
                // The number of components in our attribute ( 3 values in a Vec3 )
                3,
                // The data type
                glow::FLOAT,
                // makes integer types normalized to 0 and 1 when converting to float
                false,
                // The space between each vertex attribute and the next ( which
                // is the number of floats in the position + the number of
                // floats in the color + number of floats in the texture coordinate )
                9 * std::mem::size_of::<f32>() as i32,
                // The offset since the beginning of the buffer to look for the attribute
                0,
            );

            // Describe our vertex color attribute data format
            gl.vertex_attrib_pointer_f32(
                1,
                4,
                glow::FLOAT,
                false,
                9 * std::mem::size_of::<f32>() as i32,
                // Offset the length of the 3 floats in the position vector
                3 * std::mem::size_of::<f32>() as i32,
            );

            // Describe our vertex texture coordinate data format
            gl.vertex_attrib_pointer_f32(
                2,
                2,
                glow::FLOAT,
                false,
                9 * std::mem::size_of::<f32>() as i32,
                // Offset the length of the 3 floats in the position and the
                // four floats in the vertex color
                7 * std::mem::size_of::<f32>() as i32,
            );

            // Enable the position vertex attribute
            gl.enable_vertex_attrib_array(
                // also corresponds to `location = 0` in the vertex shader */
                0,
            );

            // Enable the color vertex attribute
            gl.enable_vertex_attrib_array(
                // also corresponds to `location = 1` in the vertex shader */
                1,
            );

            // Enable the texture coordinate vertex attribute
            gl.enable_vertex_attrib_array(2);

            let texture0 = load_and_bind_texture(gl, glow::TEXTURE0, "./assets/awesomeface.png");
            let texture1 = load_and_bind_texture(gl, glow::TEXTURE1, "./assets/wall.jpg");

            // Draw wireframe instead of solid
            // gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);

            Self {
                shader_program,
                vao,
                time_uniform,
                texture0,
                texture1,
                start_time: Instant::now(),
            }
        }
    }

    fn draw(&mut self, gl: &mut glow::Context) {
        unsafe {
            // Clear the screen
            gl.clear_color(0., 0.2, 0.2, 1.);
            gl.clear(glow::COLOR_BUFFER_BIT);

            // Make the linked shader program our current shader program used for
            // draw operations.
            gl.use_program(Some(self.shader_program));

            // Update the time uniform for our shader program
            gl.uniform_1_f32(
                Some(&self.time_uniform),
                self.start_time.elapsed().as_secs_f32(),
            );

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture0));
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture1));

            gl.uniform_1_i32(
                gl.get_uniform_location(self.shader_program, "imageTexture1")
                    .as_ref(),
                0,
            );
            gl.uniform_1_i32(
                gl.get_uniform_location(self.shader_program, "imageTexture2")
                    .as_ref(),
                1,
            );

            // Bind our VAO which contains our vertex attribute and buffer information
            gl.bind_vertex_array(Some(self.vao));

            // Draw the triangle!
            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
        }
    }
}

fn main() {
    me_learning_opengl::with_window::<Textures01>();
}

fn handle_shader_compile_errors(gl: &mut glow::Context, shader: u32) {
    unsafe {
        if !gl.get_shader_compile_status(shader) {
            eprintln!("Shader compile error: {}", gl.get_shader_info_log(shader));
            std::process::exit(1);
        }
    }
}

fn handle_program_link_errors(gl: &mut glow::Context, program: u32) {
    unsafe {
        if !gl.get_program_link_status(program) {
            eprintln!("Shader link error: {}", gl.get_program_info_log(program));
            std::process::exit(1);
        }
    }
}

fn load_and_bind_texture<P: AsRef<Path>>(gl: &mut glow::Context, unit: u32, path: P) -> u32 {
    unsafe {
        // Select the texture unit
        gl.active_texture(unit);

        // Open the texture image
        let img = image::open(path).unwrap();
        let (width, height, pixels, format) = match img {
            image::DynamicImage::ImageRgb8(img) => {
                (img.width(), img.height(), img.into_raw(), glow::RGB)
            }
            image::DynamicImage::ImageRgba8(img) => {
                (img.width(), img.height(), img.into_raw(), glow::RGBA)
            }
            _ => unimplemented!("Image format not implemented"),
        };

        // Create the GL texture for our rectangle
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        // Set our texure parameters
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );

        // Set our image data
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            format as i32,
            width as i32,
            height as i32,
            0,
            format,
            glow::UNSIGNED_BYTE,
            Some(&pixels),
        );

        // Generate mipmaps
        gl.generate_mipmap(glow::TEXTURE_2D);

        texture
    }
}
