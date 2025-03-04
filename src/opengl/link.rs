use gl::types::*;

pub unsafe fn service(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
  let program = gl::CreateProgram();
  gl::AttachShader(program, vertex_shader);
  gl::AttachShader(program, fragment_shader);
  gl::LinkProgram(program);

  let mut success = gl::FALSE as GLint;
  gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
  if success == gl::FALSE as GLint {
      let mut len = 0;
      gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
      let mut buffer = Vec::with_capacity(len as usize);
      buffer.set_len((len as usize) - 1);
      gl::GetProgramInfoLog(
          program,
          len,
          std::ptr::null_mut(),
          buffer.as_mut_ptr() as *mut GLchar,
      );
      panic!(
          "Program linking failed: {}",
          std::str::from_utf8(&buffer).unwrap()
      );
  }
  program
}