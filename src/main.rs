extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;

mod mesh;
mod scene_graph;
mod toolbox;

use scene_graph::SceneNode;

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
    
    let mut vbo1: gl::types::GLuint = 0; // Vertices
    let mut vbo2: gl::types::GLuint = 0; // Indices
    let mut vbo3: gl::types::GLuint = 0; // Rgba colors
    let mut vbo4: gl::types::GLuint = 0; // Normals

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
        // 3 * size_of::<f32>(),
        0,
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
        3 * size_of::<f32>(),
        ptr::null(),
    );
    gl::EnableVertexAttribArray(2);

    return vao;
} 

unsafe fn draw_scene(node: &scene_graph::SceneNode, view_projection_matrix: &glm::Mat4) {

    if node.index_count >= 0 {
        let mtx: glm::Mat4 = view_projection_matrix * node.current_transformation_matrix;
        gl::UniformMatrix4fv(3, 1, gl::FALSE, mtx.as_ptr());
        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(
            gl::TRIANGLES,
            node.index_count,
            gl::UNSIGNED_INT,
            ptr::null(),
        );
    }

    // Recurse
    for &child in &node.children {
        draw_scene(&*child, view_projection_matrix);
    }
}

// Task 3a // ========================================================================================================
unsafe fn update_node_transformations(node: &mut scene_graph::SceneNode, transformation_so_far: &glm::Mat4) {

    // Task 3c // ========================================================
    let mut transformation: glm::Mat4 = *transformation_so_far;

    transformation = glm::translation(&glm::vec3(-node.reference_point.x, -node.reference_point.y, -node.reference_point.z)) * transformation;
    transformation = glm::rotation(node.rotation.x, &glm::vec3(1.0, 0.0, 0.0)) * transformation;
    transformation = glm::rotation(node.rotation.y, &glm::vec3(0.0, 1.0, 0.0)) * transformation;
    transformation = glm::rotation(node.rotation.z, &glm::vec3(0.0, 0.0, 1.0)) * transformation;
    transformation = glm::translation(&glm::vec3(node.reference_point.x, node.reference_point.y, node.reference_point.z)) * transformation;

    transformation = glm::translation(&node.position) * transformation;
    // transformation = glm::scaling(&node.scale) * transformation;

    // Update the node's transformation matrix
    node.current_transformation_matrix =  transformation_so_far * transformation;
    // ===================================================================
    
    // Recurse
    for &child in &node.children {
        update_node_transformations(&mut *child, &node.current_transformation_matrix);
    }
}
// ====================================================================================================================

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

        // Load lunarsurface data
        let lunarsurface_mesh = mesh::Terrain::load(".\\resources\\lunarsurface.obj");
        let helicopter_mesh = mesh::Helicopter::load(".\\resources\\helicopter.obj");

        // Declaring vao variables of lunar landscape and the helicopter parts
        let lunarsurface_vao: u32; 

        // TASK 2a // ===============================
        let helicopter_body_vao: u32;
        let helicopter_door_vao: u32;
        let helicopter_main_rotor_vao: u32;
        let helicopter_tail_rotor_vao: u32;
        // ==========================================

        // Set up the vao's
        unsafe {
            lunarsurface_vao = setup_vao(
                &lunarsurface_mesh.vertices, 
                &lunarsurface_mesh.indices, 
                &lunarsurface_mesh.colors, 
                &lunarsurface_mesh.normals,
            );

            // TASK 2a // ===============================
            helicopter_body_vao = setup_vao(
                &helicopter_mesh[0].vertices,
                &helicopter_mesh[0].indices,
                &helicopter_mesh[0].colors,
                &helicopter_mesh[0].normals,
            );
            helicopter_door_vao = setup_vao(
                &helicopter_mesh[3].vertices,
                &helicopter_mesh[3].indices,
                &helicopter_mesh[3].colors,
                &helicopter_mesh[3].normals,
            );
            helicopter_main_rotor_vao = setup_vao(
                &helicopter_mesh[1].vertices,
                &helicopter_mesh[1].indices,
                &helicopter_mesh[1].colors,
                &helicopter_mesh[1].normals,
            );
            helicopter_tail_rotor_vao = setup_vao(
                &helicopter_mesh[2].vertices,
                &helicopter_mesh[2].indices,
                &helicopter_mesh[2].colors,
                &helicopter_mesh[2].normals,
            );
            // ============================================
        }

        // Set up scene nodes
        let mut root = SceneNode::new();
        let mut lunarsurface_node = SceneNode::from_vao(lunarsurface_vao, lunarsurface_mesh.index_count);
        let mut helicopter_root = SceneNode::new();
        let mut helicopter_body_node = SceneNode::from_vao(helicopter_body_vao, helicopter_mesh[0].index_count);
        let mut helicopter_door_node = SceneNode::from_vao(helicopter_door_vao, helicopter_mesh[3].index_count);
        let mut helicopter_main_rotor_node = SceneNode::from_vao(helicopter_main_rotor_vao, helicopter_mesh[1].index_count);
        let mut helicopter_tail_rotor_node = SceneNode::from_vao(helicopter_tail_rotor_vao, helicopter_mesh[2].index_count);


        // TASK 3b // ===============================================================
        lunarsurface_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        helicopter_body_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        helicopter_door_node.reference_point = glm::vec3(1.2, 0.0, 0.0);
        helicopter_main_rotor_node.reference_point = glm::vec3(0.0, 0.0, 0.0);
        helicopter_tail_rotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);
        // ==========================================================================

        // Adding child nodes to the helicopter body
        helicopter_body_node.add_child(&helicopter_main_rotor_node);
        helicopter_body_node.add_child(&helicopter_tail_rotor_node);
        helicopter_body_node.add_child(&helicopter_door_node);

        // Adding the entire helicopter as a child to the helicopter root node
        helicopter_root.add_child(&helicopter_body_node);

        // Adding the helicopter root node to surface node
        lunarsurface_node.add_child(&helicopter_root);

        // Adding every node to the main root node
        root.add_child(&lunarsurface_node);

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        let mut translate_vec = glm::vec3(0.0, 0.0, 0.0);
        let mut rotate_vec = glm::vec3(0.0, 0.0, 0.0);
        let mut scaling_vec = glm::vec3(1.0, 1.0, 1.0);

        // Attaches the relevant shader files and activates it
        unsafe {
            let main_shader = shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link();
            main_shader.activate();
        };

        // Perspective transformation
        let perspective_mtx: glm::Mat4 = glm::perspective(SCREEN_H as f32 / SCREEN_W as f32, 90.0, 1.0, 1000.0);

        // Transform to negative z-axis
        let perspective_translation_mtx: glm::Mat4 = glm::translation(&glm::vec3(0.0, 0.0, -1.0));

        // Projection matrix
        let projection_mtx: glm::Mat4 = perspective_mtx * perspective_translation_mtx;

        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Constructs matrix each frame
            let mut mtx: glm::Mat4 = glm::identity();

            // Transform xyz position
            let position_xyz_mtx: glm::Mat4 = glm::translation(&translate_vec);

            // Transform xy rotation
            let rotation_xy_mtx: glm::Mat4 = glm::rotation(-rotate_vec[1], &glm::vec3(1.0, 0.0, 0.0)) * glm::rotation(rotate_vec[0], &glm::vec3(0.0, 1.0, 0.0));
            
            // Combines xyz movement and rotation
            let pos_rot_mtx: glm::Mat4 = rotation_xy_mtx * position_xyz_mtx;

            // Scale transformation (Dont need this ???)
            // let scaling_mtx: glm::Mat4 = glm::scaling(&scaling_vec);

            mtx = mtx * projection_mtx * pos_rot_mtx;

            // Task 4a // ================================================
            helicopter_main_rotor_node.rotation.y = elapsed;
            helicopter_tail_rotor_node.rotation.x = elapsed;
            // ===========================================================

            // Task 4b // ================================================
            // let heading = toolbox::simple_heading_animation(elapsed);
            // helicopter_root.position.z = heading.z;
            // helicopter_root.position.x = heading.x;

            // helicopter_root.rotation.z = heading.roll;
            // helicopter_root.rotation.y = heading.yaw;
            // helicopter_root.rotation.x = heading.pitch;
            // ===========================================================

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // VirtualKeyCode::A => {
                        //     _arbitrary_number += delta_time;
                        // },
                        // VirtualKeyCode::D => {
                        //     _arbitrary_number -= delta_time;
                        // },
                        VirtualKeyCode::W => {
                            translate_vec.y -= 20.0 * delta_time;
                        },
                        VirtualKeyCode::A => {
                            translate_vec.x += 20.0 * delta_time;
                        },
                        VirtualKeyCode::D => {
                            translate_vec.x -= 20.0 * delta_time;
                        },
                        VirtualKeyCode::S => {
                            translate_vec.y += 20.0 * delta_time;
                        },
                        VirtualKeyCode::Q => {
                            translate_vec.z -= 20.0 * delta_time;
                        },
                        VirtualKeyCode::E => {
                            translate_vec.z += 20.0 * delta_time;
                        },
                        VirtualKeyCode::I => {
                            scaling_vec.z -= delta_time;
                        },
                        VirtualKeyCode::O => {
                            scaling_vec.z += delta_time;
                        },
                        VirtualKeyCode::Up => {
                            rotate_vec.y += delta_time;
                        },
                        VirtualKeyCode::Down => {
                            rotate_vec.y -= delta_time;
                        },
                        VirtualKeyCode::Right => {
                            rotate_vec.x += delta_time;
                        },
                        VirtualKeyCode::Left => {
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

                // Task 3d // ======================================================
                update_node_transformations(&mut root, &glm::Mat4::identity());
                // =================================================================
                
                // TASK 2c // ======================================================
                draw_scene(&root, &mtx);
                // =================================================================

                // TASK 2b // ================================================================================================
                // gl::BindVertexArray(lunarsurface_vao);
                // gl::DrawElements(gl::TRIANGLES, lunarsurface_mesh.index_count, gl::UNSIGNED_INT, ptr::null());

                // gl::BindVertexArray(helicopter_body_vao);
                // gl::DrawElements(gl::TRIANGLES, helicopter_mesh[0].index_count, gl::UNSIGNED_INT, ptr::null());
                
                // gl::BindVertexArray(helicopter_door_vao);
                // gl::DrawElements(gl::TRIANGLES, helicopter_mesh[1].index_count, gl::UNSIGNED_INT, ptr::null());
                
                // gl::BindVertexArray(helicopter_main_rotor_vao);
                // gl::DrawElements(gl::TRIANGLES, helicopter_mesh[2].index_count, gl::UNSIGNED_INT, ptr::null());
                
                // gl::BindVertexArray(helicopter_tail_rotor_vao);
                // gl::DrawElements(gl::TRIANGLES, helicopter_mesh[3].index_count, gl::UNSIGNED_INT, ptr::null());
                // ===========================================================================================================
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
