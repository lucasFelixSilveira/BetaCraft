use winres::WindowsResource;

fn main() {
  if cfg!(target_os = "windows") {
    WindowsResource::new()
      .set_icon("icon.ico")
      .compile()
      .expect("Failed to set icon");
  }
}