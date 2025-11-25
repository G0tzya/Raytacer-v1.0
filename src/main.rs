use glfw::{Context};
use glow::HasContext;
use glam::Vec3;

mod renderer;
mod primitives;
mod lights;

fn main() {
    let screen_width: i32 = 400;
    let screen_height: i32 = 400;

    let max_pixel_average = 40;

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(4));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, _) = glfw
        .create_window(screen_width as u32, screen_height as u32, "RAY_TRACER", glfw::WindowMode::Windowed)
        .expect("Failed to create window");
    window.make_current();
    window.set_resizable(false);
    glfw.set_swap_interval(glfw::SwapInterval::None);

    let gl = unsafe { 
        glow::Context::from_loader_function(|s| { 
            match window.get_proc_address(s) { 
                Some(p) => std::mem::transmute::<unsafe extern "C" fn(), 
                *const std::ffi::c_void>(p), None => std::ptr::null(), 
            } 
        }) 
    };
    unsafe {
        gl.viewport(0, 0, screen_width, screen_height);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
    }

    let program = unsafe {
        let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
        gl.shader_source(vs, VERTEX_SHADER);
        gl.compile_shader(vs);
        if !gl.get_shader_compile_status(vs) {
            panic!("Vertex shader compile error: {}", gl.get_shader_info_log(vs));
        }

        let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
        gl.shader_source(fs, PIXEL_SHADER);
        gl.compile_shader(fs);
        if !gl.get_shader_compile_status(fs) {
            panic!("Fragment shader compile error: {}", gl.get_shader_info_log(fs));
        }

        let prog = gl.create_program().unwrap();
        gl.attach_shader(prog, vs);
        gl.attach_shader(prog, fs);
        gl.link_program(prog);
        if !gl.get_program_link_status(prog) {
            panic!("Shader link error: {}", gl.get_program_info_log(prog));
        }

        gl.delete_shader(vs);
        gl.delete_shader(fs);

        prog
    };

    let vertices: [f32; 20] = [
        -1.0, -1.0, 0.0, 0.0, 0.0,
         1.0, -1.0, 0.0, 1.0, 0.0,
         1.0,  1.0, 0.0, 1.0, 1.0,
        -1.0,  1.0, 0.0, 0.0, 1.0,
    ];
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    let vao = unsafe { gl.create_vertex_array().unwrap() };
    let vbo = unsafe { gl.create_buffer().unwrap() };
    let ebo = unsafe { gl.create_buffer().unwrap() };

    unsafe {
        gl.bind_vertex_array(Some(vao));

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::STATIC_DRAW);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&indices), glow::STATIC_DRAW);

        let stride = (5 * std::mem::size_of::<f32>()) as i32;
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, (3 * std::mem::size_of::<f32>()) as i32);
    }

    // make texture
    let texture = unsafe{ gl.create_texture().unwrap()};
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            screen_width,
            screen_height,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            None,
        );
    }
    

    let buffer_size = (screen_width * screen_height * 4) as i32;
    let pbo = unsafe { gl.create_buffer().unwrap() };
    let map_flags = glow::MAP_WRITE_BIT | glow::MAP_PERSISTENT_BIT | glow::MAP_COHERENT_BIT;

    let main_camera = renderer::Camera {
        position: Vec3::new(0.0, 0.0, 2.0),
        look_at: Vec3::new(0.0, 0.0, 0.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        fov: 90.0,
        background_color: Vec3::new(0.0, 0.0, 0.0),
    };

    let sphere_one = primitives::primitives::Sphere {
        center: Vec3::new(0.0, 0.0, 0.0),
        radius: 1.0,
        material: primitives::primitives::Material {
            color: Vec3::new(200.0, 20.0, 200.0),
            roughness: 0.7,
            emission: Vec3::new(0.0, 0.0, 0.0),
        }     
    };

    let sphere_two = primitives::primitives::Sphere {
        center: Vec3::new(0.9, 0.5, 0.3),
        radius: 0.5 ,
        material: primitives::primitives::Material {
            color: Vec3::new(255.0, 255.0, 255.0) ,
            roughness: 0.70,
            emission: Vec3::new(0.0, 0.0, 0.0),
        }       
    };

    let ground_plane = primitives::primitives::Plane {
        point: Vec3::new(0.0, 1.0, 0.0),    
        normal: Vec3::new(0.0, -1.0, 0.0),
        material: primitives::primitives::Material {
            color: Vec3::new(255.0, 255.0, 255.0) ,
            roughness: 0.0,
            emission: Vec3::new(0.0, 0.0, 0.0),
        }   
    };

    let wall_plane = primitives::primitives::Plane {
        point: Vec3::new(0.0, 0.0, -2.0),    
        normal: Vec3::new(0.0, 0.0, 1.0),
        material: primitives::primitives::Material {
            color: Vec3::new(255.0, 255.0, 255.0) ,
            roughness: 0.9,
            emission: Vec3::new(0.0, 0.0, 0.0),
        }   
    };

    let point_light = lights::lights::Light::Point(lights::lights::PointLight {
        position: Vec3::new(-2.0, -4.0, 4.0),
        intensity: 0.4,
        color: Vec3::new(200.0, 200.0, 200.0),
    });


    let render_scene = renderer::Scene {
        objects: vec![&sphere_one, &ground_plane, &wall_plane, &sphere_two],
        lights: vec![&point_light],
    };

    unsafe {
        gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, Some(pbo));
        gl.buffer_storage(glow::PIXEL_UNPACK_BUFFER, buffer_size, None, map_flags);

        let ptr = gl.map_buffer_range(glow::PIXEL_UNPACK_BUFFER, 0, buffer_size, map_flags) as *mut u8;
        if ptr.is_null() {
            panic!("Failed to map PBO persistently!");
        }

        gl.use_program(Some(program));
        if let Some(loc) = gl.get_uniform_location(program, "uTexture") {
            gl.uniform_1_i32(Some(&loc), 0); // bind to texture unit 0
        }
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, Some(pbo));

        let w = screen_width as usize;
        let h = screen_height as usize;

        let pixel_count = screen_width * screen_height;
        let u32_ptr = ptr as *mut u32; // Cast byte-pointer to u32-pointer

        // LLVM perfers to have this inside the loop? It logicaly only need to happen once before the loop
        // But compiler optimizes much more aggresivly when its in the loop?!?! (short-lived alias)
        let slice = std::slice::from_raw_parts_mut(u32_ptr, pixel_count as usize);

        for y in 0..h {
            let row_start = y * w;
            let row = &mut slice[row_start..row_start + w];

            for x in 0..w {
                let mut total_pixel_color = Vec3::ZERO;
                for _ in 0..max_pixel_average {
                    let pixel_color = renderer::render_function(x, y, screen_width, screen_height, &main_camera, &render_scene);
                    total_pixel_color += pixel_color;
                }

                total_pixel_color /= max_pixel_average as f32;

                row[x] = pack_color(total_pixel_color);
            }
        }

        gl.tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            screen_width,
            screen_height,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::BufferOffset(0),
        );

        // Draw quad
        gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);

        window.swap_buffers();

        while !window.should_close() {
            glfw.poll_events();
        }
    }   
}

fn pack_color(c: Vec3) -> u32 {
    let gamma = 2.2;
    // gamma correction
    let corrected = Vec3::new(
        c.x.clamp(0.0, 1.0).powf(1.0 / gamma),
        c.y.clamp(0.0, 1.0).powf(1.0 / gamma),
        c.z.clamp(0.0, 1.0).powf(1.0 / gamma),
    );

    let r = (corrected.x * 255.0) as u8;
    let g = (corrected.y * 255.0) as u8;
    let b = (corrected.z * 255.0) as u8;
    let a = 255u8;

    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}

const VERTEX_SHADER: &str = r#"
#version 460 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;
out vec2 TexCoord;
void main() {
    gl_Position = vec4(aPos, 1.0);
    TexCoord = aTexCoord;
}
"#;

const PIXEL_SHADER: &str = r#"
#version 460 core
in vec2 TexCoord;
out vec4 FragColor;
uniform sampler2D uTexture;
void main() {
    FragColor = texture(uTexture, TexCoord);
}
"#;
