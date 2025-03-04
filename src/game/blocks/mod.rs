#[derive(Clone)]
pub struct Block {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub visible_faces: [bool; 6],
    pub is_dynamic: bool, // Novo campo para física
}

impl Block {
    pub fn new(id: &str, x: i32, y: i32, z: i32) -> Self {
        Block {
            id: id.to_string(),
            x,
            y,
            z,
            visible_faces: [true; 6],
            is_dynamic: false, // Padrão: blocos são estáticos
        }
    }
}