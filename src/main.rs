use gl::types::*;
use glfw::{Action, Context, Key};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

mod opengl;
mod game; 

use game::blocks::Block;
use game::Camera;

fn main() {
  println!("Current working directory: {:?}", std::env::current_dir().unwrap());

  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
  glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
  glfw.window_hint(glfw::WindowHint::OpenGlProfile(
    glfw::OpenGlProfileHint::Core,
  ));

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
    // ... (unchanged block vertex data) ...
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

    gl::VertexAttribPointer(
      0,
      3,
      gl::FLOAT,
      gl::FALSE,
      5 * std::mem::size_of::<f32>() as GLsizei,
      std::ptr::null(),
    );
    gl::EnableVertexAttribArray(0);

    gl::VertexAttribPointer(
      1,
      2,
      gl::FLOAT,
      gl::FALSE,
      5 * std::mem::size_of::<f32>() as GLsizei,
      (3 * std::mem::size_of::<f32>()) as *const _,
    );
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
    void main() {
      gl_Position = projection * view * model * vec4(aPos, 1.0);
      TexCoord = aTexCoord;
    }
  "#;

  let fragment_shader_source = r#"
    #version 330 core
    out vec4 FragColor;
    in vec2 TexCoord;
    uniform sampler2D texture1;
    void main() {
      FragColor = texture(texture1, TexCoord);
    }
  "#;

  let vertex_shader = unsafe { opengl::compile::service(vertex_shader_source, gl::VERTEX_SHADER) };
  let fragment_shader = unsafe { opengl::compile::service(fragment_shader_source, gl::FRAGMENT_SHADER) };
  let shader_program = unsafe { opengl::link::service(vertex_shader, fragment_shader) };

  unsafe {
    gl::DeleteShader(vertex_shader);
    gl::DeleteShader(fragment_shader);
  }

  unsafe {
    gl::Enable(gl::DEPTH_TEST);
    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
  }

  let mut block_textures: HashMap<String, [GLuint; 6]> = HashMap::new();
  let dirt_texture = game::load_texture("textures/dirt.png");
  block_textures.insert("minecraft:dirt".to_string(), [dirt_texture; 6]);
  
  let grass_top = game::load_texture("textures/grass/grass_top.png");
  let grass_side = game::load_texture("textures/grass/grass_side.png");
  let grass_bottom = dirt_texture;
  block_textures.insert("minecraft:grass".to_string(), [
    grass_side, grass_side, grass_top, grass_bottom, grass_side, grass_side
  ]);
  
  let stone_texture = game::load_texture("textures/stone.png");
  block_textures.insert("minecraft:stone".to_string(), [stone_texture; 6]);

  let mut blocks = vec![
    Block::new("minecraft:dirt", 0, 0, 0),
    Block::new("minecraft:grass", 0, 1, 0),
    Block::new("minecraft:dirt", 1, 0, 0),
    Block::new("minecraft:grass", 1, 1, 0),
    Block::new("minecraft:dirt", -1, 0, 0),
    Block::new("minecraft:stone", 4, 2, 0),
  ];

  fn is_solid_block(blocks: &Vec<Block>, x: i32, y: i32, z: i32) -> bool {
    blocks.iter().any(|b| b.x == x && b.y == y && b.z == z && b.id != "minecraft:air")
  }

  for i in 0..blocks.len() {
    let mut block = blocks[i].clone();
    if block.id != "minecraft:air" {
      block.visible_faces[0] = !is_solid_block(&blocks, block.x, block.y, block.z + 1);
      block.visible_faces[1] = !is_solid_block(&blocks, block.x, block.y, block.z - 1);
      block.visible_faces[2] = !is_solid_block(&blocks, block.x, block.y + 1, block.z);
      block.visible_faces[3] = !is_solid_block(&blocks, block.x, block.y - 1, block.z);
      block.visible_faces[4] = !is_solid_block(&blocks, block.x - 1, block.y, block.z);
      block.visible_faces[5] = !is_solid_block(&blocks, block.x + 1, block.y, block.z);
    }
    blocks[i] = block;
  }

  let mut camera = Camera {
    position: Vec3::new(0.0, 2.0, 5.0),
    front: Vec3::new(0.0, 0.0, -1.0),
    up: Vec3::new(0.0, 1.0, 0.0),
    yaw: -90.0,
    pitch: 0.0,
    speed: 2.5,
  };

  let mut last_x = 400.0;
  let mut last_y = 300.0;
  let mut first_mouse = true;

  let mut delta_time = 0.0;
  let mut last_frame = 0.0;
  let mut frame_count = 0;
  let mut last_fps_time = 0.0;

  // Crosshair setup (textured quad)
  let mut crosshair_vao = 0;
  let mut crosshair_vbo = 0;
  let crosshair_vertices: [f32; 20] = [
    -0.038, -0.048, 0.0, 0.0, 0.0, 
      0.038, -0.048, 0.0, 1.0, 0.0, 
      0.038,  0.048, 0.0, 1.0, 1.0, 
    -0.038,  0.048, 0.0, 0.0, 1.0,
  ];
  let crosshair_indices: [u32; 6] = [
    0, 1, 2,
    0, 2, 3,
  ];

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
    gl::BufferData(
      gl::ARRAY_BUFFER,
      (crosshair_vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
      crosshair_vertices.as_ptr() as *const _,
      gl::STATIC_DRAW,
    );

    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, crosshair_ebo);
    gl::BufferData(
      gl::ELEMENT_ARRAY_BUFFER,
      (crosshair_indices.len() * std::mem::size_of::<u32>()) as GLsizeiptr,
      crosshair_indices.as_ptr() as *const _,
      gl::STATIC_DRAW,
    );

    gl::VertexAttribPointer(
      0,
      3,
      gl::FLOAT,
      gl::FALSE,
      5 * std::mem::size_of::<f32>() as GLsizei,
      std::ptr::null(),
    );
    gl::EnableVertexAttribArray(0);

    gl::VertexAttribPointer(
      1,
      2,
      gl::FLOAT,
      gl::FALSE,
      5 * std::mem::size_of::<f32>() as GLsizei,
      (3 * std::mem::size_of::<f32>()) as *const _,
    );
    gl::EnableVertexAttribArray(1);

    let crosshair_vs = opengl::compile::service(crosshair_vertex_shader, gl::VERTEX_SHADER);
    let crosshair_fs = opengl::compile::service(crosshair_fragment_shader, gl::FRAGMENT_SHADER);
    crosshair_program = opengl::link::service(crosshair_vs, crosshair_fs);

    gl::DeleteShader(crosshair_vs);
    gl::DeleteShader(crosshair_fs);

    // Enable alpha blending for transparency
    gl::Enable(gl::BLEND);
    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
  }

  while !window.should_close() {
    let current_frame = glfw.get_time() as f32;
    delta_time = current_frame - last_frame;
    last_frame = current_frame;

    frame_count += 1;
    if current_frame - last_fps_time >= 1.0 {
      let fps = frame_count as f32 / (current_frame - last_fps_time);
      window.set_title(&format!("Mini Minecraft - FPS: {:.2}", fps));
      frame_count = 0;
      last_fps_time = current_frame;
    }

    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
      match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
          window.set_should_close(true);
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
          camera.yaw += xoffset * sensitivity;
          camera.pitch += yoffset * sensitivity;

          camera.pitch = camera.pitch.clamp(-89.0, 89.0);

          let direction = Vec3::new(
            camera.yaw.to_radians().cos() * camera.pitch.to_radians().cos(),
            camera.pitch.to_radians().sin(),
            camera.yaw.to_radians().sin() * camera.pitch.to_radians().cos(),
          );
          camera.front = direction.normalize();
        }
        _ => {}
      }
    }

    let camera_speed = camera.speed * delta_time;
    if window.get_key(Key::W) == Action::Press {
      camera.position += camera.front * camera_speed;
    }

    if window.get_key(Key::S) == Action::Press {
      camera.position -= camera.front * camera_speed;
    }

    if window.get_key(Key::A) == Action::Press {
      camera.position -= camera.front.cross(camera.up).normalize() * camera_speed;
    }

    if window.get_key(Key::D) == Action::Press {
      camera.position += camera.front.cross(camera.up).normalize() * camera_speed;
    }

    unsafe {
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
      gl::UseProgram(shader_program);
      gl::BindVertexArray(vao);

      let view = Mat4::look_at_rh(
        camera.position,
        camera.position + camera.front,
        camera.up,
      );

      let projection = Mat4::perspective_rh_gl(
        45.0_f32.to_radians(),
        800.0 / 600.0,
        0.1,
        100.0,
      );

      let view_loc = gl::GetUniformLocation(shader_program, "view\0".as_ptr() as *const _);
      let proj_loc = gl::GetUniformLocation(shader_program, "projection\0".as_ptr() as *const _);
      gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ref().as_ptr());
      gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ref().as_ptr());

      for block in &blocks {
        let model = Mat4::from_scale(Vec3::new(block_size, block_size, block_size)) * 
              Mat4::from_translation(Vec3::new(
                block.x as f32 * block_size,
                block.y as f32 * block_size,
                block.z as f32 * block_size,
              ));

        let model_loc = gl::GetUniformLocation(shader_program, "model\0".as_ptr() as *const _);
        gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ref().as_ptr());

        let textures = block_textures.get(&block.id).unwrap_or(&block_textures["minecraft:dirt"]);
        for face in 0..6 {
          if block.visible_faces[face] {
            gl::BindTexture(gl::TEXTURE_2D, textures[face]);
            gl::DrawArrays(gl::TRIANGLES, (face * 6) as GLint, 6);
          }
        }
      }

      // Draw textured crosshair
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