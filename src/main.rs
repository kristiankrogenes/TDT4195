extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod mesh;
mod scene_graph;
mod toolbox;
mod shader;
mod util;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Modify and complete the function below for the first task
unsafe fn setup_vao(vertices: &Vec<f32>, indices: &Vec<u32>, rgba: &Vec<f32>, normals: &Vec<f32>) -> u32 { 

    let mut vao: gl::types::GLuint = 0;
    
    let mut vbo1: gl::types::GLuint = 0;
    let mut vbo2: gl::types::GLuint = 0;
    let mut vbo3: gl::types::GLuint = 0;
    let mut vbo4: gl::types::GLuint = 0;

    gl::GenVertexArrays(1, &mut vao);
    assert!(vao != 0);
    gl::BindVertexArray(vao);

    gl::GenBuffers(1, &mut vbo1);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo1);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&vertices),
        pointer_to_array(&vertices),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        3 * size_of::<f32>(),
        ptr::null(),
    );
    gl::EnableVertexAttribArray(0);

    gl::GenBuffers(1, &mut vbo2);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo2);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(&indices),
        pointer_to_array(&indices),
        gl::STATIC_DRAW,
    );

    gl::GenBuffers(1, &mut vbo3);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo3);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&rgba),
        pointer_to_array(&rgba),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(
        1,
        4,
        gl::FLOAT,
        gl::FALSE,
        4 * size_of::<f32>(),
        ptr::null(),
    );
    gl::EnableVertexAttribArray(1);

    gl::GenBuffers(1, &mut vbo4);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo4);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&normals),
        pointer_to_array(&normals),
        gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(
        2,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 3,
        ptr::null(),
    );
    gl::EnableVertexAttribArray(2);

    return vao;
} 

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // Basic usage of shader helper:
        // The example code below returns a shader object, which contains the field `.program_id`.
        // The snippet is not enough to do the assignment, and will need to be modified (outside of
        // just using the correct path), but it only needs to be called once
        //
        //     shader::ShaderBuilder::new()
        //        .attach_file("./path/to/shader.file")
        //        .link();

        // Load lunarsurface data
        let lunarsurface_mesh = mesh::Terrain::load(".\\resources\\lunarsurface.obj");

        // Setting up lunar vao
        let lunarsurface_vao: u32; 

        unsafe {
            lunarsurface_vao = setup_vao(
                &lunarsurface_mesh.vertices, 
                &lunarsurface_mesh.indices, 
                &lunarsurface_mesh.colors, 
                &lunarsurface_mesh.normals,
            );
        };

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        let mut translate_vec = glm::vec3(0.0, 0.0, 0.0);
        let mut rotate_vec = glm::vec3(0.0, 0.0, 0.0);
        let mut scaling_vec = glm::vec3(1.0, 1.0, 1.0);

        unsafe {
            let main_shader = shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link();
            main_shader.activate();
        };

        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Constructs matrix each fram
            let mut mtx: glm::Mat4 = glm::identity();

            // let model: glm::Mat4 = glm::perspective((SCREEN_H as f32) / (SCREEN_W as f32), 90.0, 1.0, 1000.0);

            // Transform to negative z-axis
            let translation_mtx: glm::Mat4 = glm::translation(&glm::vec3(0.0, 0.0, -1.0));

            // let proj: glm::Mat4 = model * translation_mtx;

            // Transform xyz position
            let position_xyz_mtx: glm::Mat4 = glm::translation(&translate_vec);

            // Transform xy rotation
            let rotation_xy_mtx: glm::Mat4 = glm::rotation(-rotate_vec[1], &glm::vec3(1.0, 0.0, 0.0)) * glm::rotation(rotate_vec[0], &glm::vec3(0.0, 1.0, 0.0));
            
            // Combines xyz movement and rotation
            let pos_rot_mtx: glm::Mat4 = rotation_xy_mtx * position_xyz_mtx;

            // Scale transformation
            let scaling_mtx: glm::Mat4 = glm::scaling(&scaling_vec);

            // Projection transformation
            let projection_mtx: glm::Mat4 = glm::perspective(SCREEN_H as f32 / SCREEN_W as f32, 0.5, 1.0, 100.0);

            mtx = mtx * projection_mtx * pos_rot_mtx * scaling_mtx * mtx;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                        },
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        },
                        VirtualKeyCode::W => {
                            translate_vec.y += delta_time;
                        },
                        VirtualKeyCode::A => {
                            translate_vec.x -= delta_time;
                        },
                        VirtualKeyCode::D => {
                            translate_vec.x += delta_time;
                        },
                        VirtualKeyCode::S => {
                            translate_vec.y -= delta_time;
                        },
                        VirtualKeyCode::Q => {
                            scaling_vec.z -= delta_time;
                        },
                        VirtualKeyCode::E => {
                            scaling_vec.z += delta_time;
                        },
                        VirtualKeyCode::Z => {
                            rotate_vec.y += delta_time;
                        },
                        VirtualKeyCode::X => {
                            rotate_vec.y -= delta_time;
                        },
                        VirtualKeyCode::C => {
                            rotate_vec.x += delta_time;
                        },
                        VirtualKeyCode::V => {
                            rotate_vec.x -= delta_time;
                        },

                        _ => { }
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                *delta = (0.0, 0.0);
            }

            unsafe {
                gl::ClearColor(0.76862745, 0.71372549, 0.94901961, 1.0); // moon raker, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue the necessary commands to draw your scene here
                gl::BindVertexArray(lunarsurface_vao);
                gl::UniformMatrix4fv(3, 1, gl::FALSE, mtx.as_ptr());

                gl::DrawElements(gl::TRIANGLES, lunarsurface_mesh.index_count, gl::UNSIGNED_INT, ptr::null());
            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}
