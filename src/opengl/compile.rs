use gl::types::*;

pub unsafe fn service(source: &str, shader_type: GLenum) -> GLuint {
  let shader = gl::CreateShader(shader_type);
  let c_str = std::ffi::CString::new(source).unwrap();
  gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
  gl::CompileShader(shader);

  let mut success = gl::FALSE as GLint;
  gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
  if success == gl::FALSE as GLint {
    let mut len = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    let mut buffer = Vec::with_capacity(len as usize);
    buffer.set_len((len as usize) - 1);
    gl::GetShaderInfoLog(
      shader,
      len,
      std::ptr::null_mut(),
      buffer.as_mut_ptr() as *mut GLchar,
    );
    panic!(
      "Shader compilation failed: {}",
      std::str::from_utf8(&buffer).unwrap()
    );
  }
  shader
}