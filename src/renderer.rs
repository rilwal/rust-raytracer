
extern crate glfw;
extern crate gl;
extern crate glm;

// TODO: make the render target and window size independent

use std::{ffi::c_void, mem::size_of};
use glfw::{Glfw, Action, Context, Key};


#[derive(Copy, Clone, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn color(r: u8, g: u8, b: u8) -> Color {
    Color{r:r, g:g, b:b}
}


const VERT_SHADER_SRC : &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;

out vec2 vertPos;

void main()
{
    vertPos = vec2(aPos.x, aPos.y);
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}
"#;


const FRAG_SHADER_SRC : &str = r#"
#version 330 core
out vec4 FragColor;
uniform sampler2D tex;

in vec2 vertPos;

void main()
{
    vec3 rgb = texture(tex, vertPos / 2 + .5).rgb;
    FragColor = vec4(rgb, 1.0f);
}
"#;

pub struct Renderer {
    glfw : Option<Glfw>,
    window : Option<glfw::Window>,
    width: u32,
    height: u32,
    events : Option<std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>>,
    pub image_data : Vec<Color>,
    vao: u32,
    vbo: u32,
    texture: u32,
    program: u32,
}

impl Renderer {
    pub fn create(width : u32, height : u32) -> Self {
        Self {
            glfw: None,
            window: None,
            events: None,
            width: width,
            height: height,
            image_data: Vec::with_capacity((width * height) as usize),
            vao: 0,
            vbo: 0,
            texture: 0,
            program: 0
        }
    }


    pub fn initialize(&mut self) {      
        self.glfw = Some(glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to init GLFW"));
        
        let glfw = self.glfw.as_mut().unwrap();

        let (window, events) = glfw.create_window(self.width, self.height, "Rusty Raytracer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    
        self.window = Some(window);
        self.events = Some(events);

        let window = self.window.as_mut().unwrap();
    
        window.set_key_polling(true);
        window.make_current();
    
        gl::load_with(|s| window.get_proc_address(s) as *const _);
    
        self.image_data.resize((self.width * self.height) as usize, Color{r: 0, g: 0, b: 0});
    
        // And model data
        let mut model_data = [glm::vec3(-1.0, -1.0, 0.0), glm::vec3(1.0, -1.0, 0.0), glm::vec3(-1.0, 1.0, 0.0),  glm::vec3(1.0, -1.0, 0.0), glm::vec3(-1.0, 1.0, 0.0), glm::vec3(1.0, 1.0, 0.0)];
        
        unsafe {
            // Setup a texture
            gl::GenTextures(1, &mut self.texture);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
    
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
    
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, self.width as i32, self.height as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, self.image_data.as_mut_ptr() as *const c_void);
    
            // Setup a VAO
            gl::GenVertexArrays(1, &mut self.vao);
            gl::BindVertexArray(self.vao);
            let vptr_model = model_data.as_mut_ptr() as *mut c_void;
    
            // and the vertex buffer object
            gl::GenBuffers(1, &mut self.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (std::mem::size_of::<glm::Vec3>() * model_data.len()) as isize, vptr_model, gl::STATIC_DRAW);
    
            // and setup the format of the vertex data
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<f32>() as i32, 0 as *const c_void);
            gl::EnableVertexAttribArray(0);
    
            // and unset the VAO
            gl::BindVertexArray(0);
    
            // and lastly, let's setup some shaders!
            let vert_shader_id = gl::CreateShader(gl::VERTEX_SHADER);
            let frag_shader_id = gl::CreateShader(gl::FRAGMENT_SHADER);
    
            let vert_shader_src_ptr = VERT_SHADER_SRC.as_ptr() as *const i8;
            let vert_shader_len = VERT_SHADER_SRC.len() as i32;
    
            let frag_shader_src_ptr = FRAG_SHADER_SRC.as_ptr() as *const i8;
            let frag_shader_len = FRAG_SHADER_SRC.len() as i32;
    
            gl::ShaderSource(vert_shader_id, 1, &vert_shader_src_ptr as *const *const i8, &vert_shader_len); 
            gl::CompileShader(vert_shader_id);
    
            gl::ShaderSource(frag_shader_id, 1, &frag_shader_src_ptr as *const *const i8, &frag_shader_len); 
            gl::CompileShader(frag_shader_id);
    
            // and the shader program
            self.program = gl::CreateProgram();
            gl::AttachShader(self.program, vert_shader_id);
            gl::AttachShader(self.program, frag_shader_id);
    
            gl::LinkProgram(self.program);
        }
    }
 
    pub fn update(&mut self) {
        let glfw = self.glfw.as_mut().unwrap();
        let events = self.events.as_ref().unwrap();
        let window = self.window.as_mut().unwrap();

        let (w, h) = window.get_framebuffer_size();

        // Render the texture here
        unsafe {
            gl::Viewport(0, 0, w, h);

            gl::BindVertexArray(self.vao); // reload all our settings
            gl::UseProgram(self.program);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            // Upload texture to GPU
            // TODO: Track if this really changed, save time by only updating when necessary!
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, self.width as i32, self.height as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, self.image_data.as_mut_ptr() as *const c_void);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                _ => {}
            }
        }
    }

    pub fn should_close(&mut self) -> bool {
        return self.window.as_ref().unwrap().should_close();
    }

    pub fn set_pixel(&mut self, x : u32, y : u32, c : &Color) {
        self.image_data[(y * self.width as u32 + x) as usize] = *c;
    }
}

