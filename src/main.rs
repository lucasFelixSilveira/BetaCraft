use gl::types::*;
use glfw::{Action, Context, Key};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use rand::Rng;

mod opengl;
mod game;

use game::blocks::Block;
use game::{Player, PlayerInput};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    let (mut window, events) = glfw
        .create_window(800, 600, "Mini Minecraft", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let block_size: f32 = 1.0;

    let face_vertices: [[f32; 30]; 6] = [
        [-0.5, -0.5,  0.5, 0.0, 0.0,  0.5, -0.5,  0.5, 1.0, 0.0,  0.5,  0.5,  0.5, 1.0, 1.0,
         -0.5, -0.5,  0.5, 0.0, 0.0,  0.5,  0.5,  0.5, 1.0, 1.0, -0.5,  0.5,  0.5, 0.0, 1.0],
        [-0.5, -0.5, -0.5, 0.0, 0.0,  0.5, -0.5, -0.5, 1.0, 0.0,  0.5,  0.5, -0.5, 1.0, 1.0,
         -0.5, -0.5, -0.5, 0.0, 0.0,  0.5,  0.5, -0.5, 1.0, 1.0, -0.5,  0.5, -0.5, 0.0, 1.0],
        [-0.5,  0.5, -0.5, 0.0, 0.0,  0.5,  0.5, -0.5, 1.0, 0.0,  0.5,  0.5,  0.5, 1.0, 1.0,
         -0.5,  0.5, -0.5, 0.0, 0.0,  0.5,  0.5,  0.5, 1.0, 1.0, -0.5,  0.5,  0.5, 0.0, 1.0],
        [-0.5, -0.5, -0.5, 0.0, 0.0,  0.5, -0.5, -0.5, 1.0, 0.0,  0.5, -0.5,  0.5, 1.0, 1.0,
         -0.5, -0.5, -0.5, 0.0, 0.0,  0.5, -0.5,  0.5, 1.0, 1.0, -0.5, -0.5,  0.5, 0.0, 1.0],
        [-0.5, -0.5, -0.5, 0.0, 0.0, -0.5, -0.5,  0.5, 1.0, 0.0, -0.5,  0.5,  0.5, 1.0, 1.0,
         -0.5, -0.5, -0.5, 0.0, 0.0, -0.5,  0.5,  0.5, 1.0, 1.0, -0.5,  0.5, -0.5, 0.0, 1.0],
        [ 0.5, -0.5, -0.5, 0.0, 0.0,  0.5, -0.5,  0.5, 1.0, 0.0,  0.5,  0.5,  0.5, 1.0, 1.0,
          0.5, -0.5, -0.5, 0.0, 0.0,  0.5,  0.5,  0.5, 1.0, 1.0,  0.5,  0.5, -0.5, 0.0, 1.0],
    ];

    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (face_vertices[0].len() * 6 * std::mem::size_of::<f32>()) as GLsizeiptr,
            face_vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<f32>() as GLsizei, std::ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<f32>() as GLsizei, (3 * std::mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
    }

    let vertex_shader_source = r#"
        #version 330 core
        layout(location = 0) in vec3 aPos;
        layout(location = 1) in vec2 aTexCoord;
        uniform mat4 model;
        uniform mat4 view;
        uniform mat4 projection;
        out vec2 TexCoord;
        out vec3 FragPos;
        out vec3 Normal;
        void main() {
            gl_Position = projection * view * model * vec4(aPos, 1.0);
            TexCoord = aTexCoord;
            FragPos = vec3(model * vec4(aPos, 1.0));
            if (gl_VertexID < 6) Normal = vec3(0.0, 0.0, 1.0);
            else if (gl_VertexID < 12) Normal = vec3(0.0, 0.0, -1.0);
            else if (gl_VertexID < 18) Normal = vec3(0.0, 1.0, 0.0);
            else if (gl_VertexID < 24) Normal = vec3(0.0, -1.0, 0.0);
            else if (gl_VertexID < 30) Normal = vec3(-1.0, 0.0, 0.0);
            else Normal = vec3(1.0, 0.0, 0.0);
        }
    "#;

    let fragment_shader_source = r#"
        #version 330 core
        out vec4 FragColor;
        in vec2 TexCoord;
        in vec3 FragPos;
        in vec3 Normal;
        uniform sampler2D texture1;
        uniform float opacity = 1.0;
        uniform vec3 lightPos;
        uniform vec3 lightColor;
        void main() {
            vec4 texColor = texture(texture1, TexCoord);
            float ambientStrength = 0.3;
            vec3 ambient = ambientStrength * lightColor;
            vec3 norm = normalize(Normal);
            vec3 lightDir = normalize(lightPos - FragPos);
            float diff = max(dot(norm, lightDir), 0.0);
            vec3 diffuse = diff * lightColor;
            vec3 result = (ambient + diffuse) * texColor.rgb;
            FragColor = vec4(result, texColor.a * opacity);
        }
    "#;

    let vertex_shader = unsafe { opengl::compile::service(vertex_shader_source, gl::VERTEX_SHADER) };
    let fragment_shader = unsafe { opengl::compile::service(fragment_shader_source, gl::FRAGMENT_SHADER) };
    let shader_program = unsafe { opengl::link::service(vertex_shader, fragment_shader) };

    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.2, 0.4, 0.8, 1.0);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut block_textures: HashMap<String, [GLuint; 6]> = HashMap::new();
    let dirt_texture = game::load_texture("textures/dirt.png");
    block_textures.insert("minecraft:dirt".to_string(), [dirt_texture; 6]);
    let grass_top = game::load_texture("textures/grass/grass_top.png");
    let grass_side = game::load_texture("textures/grass/grass_side.png");
    block_textures.insert("minecraft:grass_block".to_string(), [grass_side, grass_side, grass_top, dirt_texture, grass_side, grass_side]);
    let stone_texture = game::load_texture("textures/stone.png");
    block_textures.insert("minecraft:stone".to_string(), [stone_texture; 6]);
    let water_texture = game::load_texture("textures/water.png");
    block_textures.insert("minecraft:water".to_string(), [water_texture; 6]);
    let oak_log_vertical = game::load_texture("textures/oak_log/oak_log_vertical.png");
    let oak_log_sides = game::load_texture("textures/oak_log/oak_log_side.png");
    block_textures.insert("minecraft:oak_log".to_string(), [oak_log_sides, oak_log_sides, oak_log_vertical, oak_log_vertical, oak_log_sides, oak_log_sides]);
    let oak_leaves_texture = game::load_texture("textures/oak_leaves.png");
    block_textures.insert("minecraft:oak_leaves".to_string(), [oak_leaves_texture; 6]);

    let (mut blocks, spawn_point) = game::world::generation::assembly(16, 16);
    update_visible_faces(&mut blocks); // Calcula faces visíveis após geração

    let mut player = Player {
        position: Vec3::new(spawn_point.0 as f32, spawn_point.1 as f32, spawn_point.2 as f32),
        velocity: Vec3::ZERO,
        size: Vec3::new(0.6, 1.8, 0.6),
        on_ground: false,
        yaw: -90.0,
        pitch: 0.0,
        front: Vec3::new(0.0, 0.0, -1.0),
        up: Vec3::new(0.0, 1.0, 0.0),
        speed: 4.317,
    };

    for block in &mut blocks {
        block.is_dynamic = block.id == "minecraft:sand"; 
    }

    fn is_solid_block(blocks: &Vec<Block>, x: i32, y: i32, z: i32) -> bool {
        blocks.iter().any(|b| b.x == x && b.y == y && b.z == z && b.id != "minecraft:water")
    }

    fn update_visible_faces(blocks: &mut Vec<Block>) {
        let mut block_positions: HashMap<(i32, i32, i32), String> = HashMap::new();
        for block in blocks.iter() {
            block_positions.insert((block.x, block.y, block.z), block.id.clone());
        }
    
        for block in blocks.iter_mut() {
            block.visible_faces = [
                !block_positions.contains_key(&(block.x, block.y, block.z + 1))
                    || block_positions[&(block.x, block.y, block.z + 1)] == "minecraft:water",
                !block_positions.contains_key(&(block.x, block.y, block.z - 1))
                    || block_positions[&(block.x, block.y, block.z - 1)] == "minecraft:water",
                !block_positions.contains_key(&(block.x, block.y + 1, block.z))
                    || block_positions[&(block.x, block.y + 1, block.z)] == "minecraft:water",
                !block_positions.contains_key(&(block.x, block.y - 1, block.z))
                    || block_positions[&(block.x, block.y - 1, block.z)] == "minecraft:water",
                !block_positions.contains_key(&(block.x - 1, block.y, block.z))
                    || block_positions[&(block.x - 1, block.y, block.z)] == "minecraft:water",
                !block_positions.contains_key(&(block.x + 1, block.y, block.z))
                    || block_positions[&(block.x + 1, block.y, block.z)] == "minecraft:water",
            ];
        }
    }

    fn update_blocks(blocks: &mut Vec<Block>, delta_time: f32) {
        const GRAVITY: f32 = -9.81;
        for i in 0..blocks.len() {
            if blocks[i].is_dynamic {
                let mut block = blocks[i].clone();
                let mut velocity_y = 0.0;
                velocity_y += GRAVITY * delta_time;
                let new_y = block.y as f32 + velocity_y * delta_time;
                if !is_solid_block(blocks, block.x, (new_y - 0.5) as i32, block.z) {
                    block.y = new_y.round() as i32;
                    blocks[i] = block;
                } else {
                    blocks[i].is_dynamic = false;
                }
            }
        }
    }

    let mut last_x = 400.0;
    let mut last_y = 300.0;
    let mut first_mouse = true;

    let mut delta_time = 0.0;
    let mut last_frame = 0.0;
    let mut input = PlayerInput::default();

    let mut crosshair_vao = 0;
    let mut crosshair_vbo = 0;
    let crosshair_vertices: [f32; 20] = [
        -0.038, -0.048, 0.0, 0.0, 0.0,
         0.038, -0.048, 0.0, 1.0, 0.0,
         0.038,  0.048, 0.0, 1.0, 1.0,
        -0.038,  0.048, 0.0, 0.0, 1.0,
    ];
    let crosshair_indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

    let crosshair_vertex_shader = r#"
        #version 330 core
        layout(location = 0) in vec3 aPos;
        layout(location = 1) in vec2 aTexCoord;
        out vec2 TexCoord;
        void main() {
            gl_Position = vec4(aPos, 1.0);
            TexCoord = aTexCoord;
        }
    "#;

    let crosshair_fragment_shader = r#"
        #version 330 core
        out vec4 FragColor;
        in vec2 TexCoord;
        uniform sampler2D texture1;
        void main() {
            FragColor = texture(texture1, TexCoord);
        }
    "#;

    let mut crosshair_ebo = 0;
    let crosshair_program;
    let crosshair_texture = game::load_texture("textures/crosshair.png");
    unsafe {
        gl::GenVertexArrays(1, &mut crosshair_vao);
        gl::GenBuffers(1, &mut crosshair_vbo);
        gl::GenBuffers(1, &mut crosshair_ebo);

        gl::BindVertexArray(crosshair_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, crosshair_vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (crosshair_vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr, crosshair_vertices.as_ptr() as *const _, gl::STATIC_DRAW);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, crosshair_ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (crosshair_indices.len() * std::mem::size_of::<u32>()) as GLsizeiptr, crosshair_indices.as_ptr() as *const _, gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<f32>() as GLsizei, std::ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<f32>() as GLsizei, (3 * std::mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);

        let crosshair_vs = opengl::compile::service(crosshair_vertex_shader, gl::VERTEX_SHADER);
        let crosshair_fs = opengl::compile::service(crosshair_fragment_shader, gl::FRAGMENT_SHADER);
        crosshair_program = opengl::link::service(crosshair_vs, crosshair_fs);

        gl::DeleteShader(crosshair_vs);
        gl::DeleteShader(crosshair_fs);
    }

    let light_pos = Vec3::new(5.0, 5.0, 5.0);
    let light_color = Vec3::new(1.0, 1.0, 1.0);

    while !window.should_close() {
      let current_frame = glfw.get_time() as f32;
      delta_time = current_frame - last_frame;
      last_frame = current_frame;

      glfw.poll_events();
      for (_, event) in glfw::flush_messages(&events) {
          match event {
              glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                  window.set_should_close(true);
              }
              glfw::WindowEvent::Key(Key::W, _, action, _) => {
                  println!("W Apertado");
                  input.forward = action == Action::Press || action == Action::Repeat;
              }
              glfw::WindowEvent::Key(Key::S, _, action, _) => {
                  println!("S Apertado");
                  input.backward = action == Action::Press || action == Action::Repeat;
              }
              glfw::WindowEvent::Key(Key::A, _, action, _) => {
                  println!("A Apertado");
                  input.left = action == Action::Press || action == Action::Repeat;
              }
              glfw::WindowEvent::Key(Key::D, _, action, _) => {
                  println!("D Apertado");
                  input.right = action == Action::Press || action == Action::Repeat;
              }
              glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                  input.jump = true;
              }
              glfw::WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                  input.jump = false;
              }
              glfw::WindowEvent::CursorPos(xpos, ypos) => {
                  let (xpos, ypos) = (xpos as f32, ypos as f32);
                  if first_mouse {
                      last_x = xpos;
                      last_y = ypos;
                      first_mouse = false;
                  }
                  let xoffset = xpos - last_x;
                  let yoffset = last_y - ypos;
                  last_x = xpos;
                  last_y = ypos;

                  let sensitivity = 0.1;
                  player.yaw += xoffset * sensitivity;
                  player.pitch += yoffset * sensitivity;
                  player.pitch = player.pitch.clamp(-89.0, 89.0);

                  let direction = Vec3::new(
                      player.yaw.to_radians().cos() * player.pitch.to_radians().cos(),
                      player.pitch.to_radians().sin(),
                      player.yaw.to_radians().sin() * player.pitch.to_radians().cos(),
                  );
                  player.front = direction.normalize();
              }
              _ => {}
          }
      }

        // Atualizar o player com física e movimentação WASD
        player.update(&blocks, delta_time, &input);
        update_blocks(&mut blocks, delta_time);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);

            let view = Mat4::look_at_rh(
                player.position + Vec3::new(0.0, player.size.y * 0.8, 0.0),
                player.position + Vec3::new(0.0, player.size.y * 0.8, 0.0) + player.front,
                player.up,
            );

            let projection = Mat4::perspective_rh_gl(45.0_f32.to_radians(), 800.0 / 600.0, 0.1, 100.0);

            let view_loc = gl::GetUniformLocation(shader_program, "view\0".as_ptr() as *const _);
            let proj_loc = gl::GetUniformLocation(shader_program, "projection\0".as_ptr() as *const _);
            gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ref().as_ptr());
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ref().as_ptr());

            let light_pos_loc = gl::GetUniformLocation(shader_program, "lightPos\0".as_ptr() as *const _);
            let light_color_loc = gl::GetUniformLocation(shader_program, "lightColor\0".as_ptr() as *const _);
            gl::Uniform3fv(light_pos_loc, 1, light_pos.as_ref().as_ptr());
            gl::Uniform3fv(light_color_loc, 1, light_color.as_ref().as_ptr());

            for block in &blocks {
                let model = Mat4::from_scale(Vec3::new(block_size, block_size, block_size)) *
                            Mat4::from_translation(Vec3::new(block.x as f32, block.y as f32, block.z as f32));
                let model_loc = gl::GetUniformLocation(shader_program, "model\0".as_ptr() as *const _);
                gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ref().as_ptr());

                let textures = block_textures.get(&block.id).unwrap_or(&block_textures["minecraft:dirt"]);
                let opacity_loc = gl::GetUniformLocation(shader_program, "opacity\0".as_ptr() as *const _);
                let opacity = if block.id == "minecraft:water" { 0.5 } else { 1.0 };
                gl::Uniform1f(opacity_loc, opacity);

                for face in 0..6 {
                    if block.visible_faces[face] {
                        gl::BindTexture(gl::TEXTURE_2D, textures[face]);
                        gl::DrawArrays(gl::TRIANGLES, (face * 6) as GLint, 6);
                    }
                }
            }

            gl::Disable(gl::DEPTH_TEST);
            gl::UseProgram(crosshair_program);
            gl::BindVertexArray(crosshair_vao);
            gl::BindTexture(gl::TEXTURE_2D, crosshair_texture);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            gl::Enable(gl::DEPTH_TEST);
        }

        window.swap_buffers();
    }

    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteProgram(shader_program);
        gl::DeleteVertexArrays(1, &crosshair_vao);
        gl::DeleteBuffers(1, &crosshair_vbo);
        gl::DeleteBuffers(1, &crosshair_ebo);
        gl::DeleteProgram(crosshair_program);
    }
}