use glow::HasContext;
use me_learning_opengl::{RenderHandler, SliceAsBytes};

const VERTEX_SHADER_SRC: &str = include_str!("hello_triangle/vertex.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("hello_triangle/fragment.glsl");
const TRI_VERTICES: &[f32] = &[-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];

struct HelloTriangle {
    /// A compiled and linked shader program: Combines the vertex shader and the
    /// fragment shader into a usable shader program.
    shader_program: u32,
    /// Vertex Array Object: It's like a vertex attributes configuration
    /// "preset"
    vao: u32,
}

impl RenderHandler for HelloTriangle {
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

            // Describe our vertex attribute data format
            gl.vertex_attrib_pointer_f32(
                // Corresponds to `location = 0` in the vertex shader
                0,
                // The number of components in our attribute ( 3 values in a Vec3 )
                3,
                // The data type
                glow::FLOAT,
                // makes integer types normalized to 0 and 1 when converting to float
                false,
                // The space between each vertex attribute and the next
                3 * std::mem::size_of::<f32>() as i32,
                // The offset since the beginning of the buffer to look for the attribute
                0,
            );

            // Enable the position vertex attribute
            gl.enable_vertex_attrib_array(
                // also corresponds to `location = 0` in the vertex shader */
                0,
            );

            // Draw wireframe instead of solid
            // gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);

            Self {
                shader_program,
                vao,
            }
        }
    }

    fn draw(&mut self, gl: &mut glow::Context) {
        unsafe {
            // Clear the screen
            gl.clear_color(0., 0.8, 0.8, 1.);
            gl.clear(glow::COLOR_BUFFER_BIT);

            // Make the linked shader program our current shader program used for
            // draw operations.
            gl.use_program(Some(self.shader_program));
            gl.bind_vertex_array(Some(self.vao));

            // Draw the triangle!
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

fn main() {
    me_learning_opengl::with_window::<HelloTriangle>();
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
