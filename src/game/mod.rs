use gl::types::*;
use glam::Vec3;

pub mod blocks;

pub struct Camera {
  pub position: Vec3,
  pub front: Vec3,
  pub up: Vec3,
  pub yaw: f32,
  pub pitch: f32,
  pub speed: f32,
}

pub fn load_texture(path: &str) -> GLuint {
  let mut texture_id = 0;
  unsafe {
    gl::GenTextures(1, &mut texture_id);
    gl::BindTexture(gl::TEXTURE_2D, texture_id);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

    match image::open(path) {
      Ok(img) => {
        let img = img.flipv();
        let data = img.to_rgba8();
        gl::TexImage2D(
          gl::TEXTURE_2D,
          0,
          gl::RGBA as GLint,
          data.width() as GLint,
          data.height() as GLint,
          0,
          gl::RGBA,
          gl::UNSIGNED_BYTE,
          data.as_ptr() as *const _,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
      }
      Err(e) => {
        eprintln!("Failed to load texture '{}': {:?}", path, e);
        gl::TexImage2D(
          gl::TEXTURE_2D,
          0,
          gl::RGBA as GLint,
          1,
          1,
          0,
          gl::RGBA,
          gl::UNSIGNED_BYTE,
          &[255, 0, 255, 255] as *const _ as *const _, 
        );
      }
    }
  }
  texture_id
}