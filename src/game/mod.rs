use gl::types::*;
use blocks::Block;
use glam::{Vec3, vec3};

pub mod blocks;
pub mod world;

#[derive(Clone)]
pub struct Player {
    pub position: Vec3,    // Posição do jogador (centro da base)
    pub velocity: Vec3,    // Velocidade em blocos/s
    pub size: Vec3,       // Tamanho do AABB (0.6 largura, 1.8 altura)
    pub on_ground: bool,  // Está no chão?
    pub yaw: f32,         // Rotação horizontal em graus
    pub pitch: f32,       // Rotação vertical em graus
    pub front: Vec3,      // Direção da câmera
    pub up: Vec3,         // Vetor "cima"
    pub speed: f32,       // Velocidade base de movimento (blocos/s)
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: vec3(16.0, 70.0, 16.0),
            velocity: Vec3::ZERO,
            size: vec3(0.6, 1.8, 0.6),
            on_ground: false,
            yaw: -90.0,
            pitch: 0.0,
            front: vec3(0.0, 0.0, -1.0),
            up: Vec3::Y,
            speed: 4.317,
        }
    }

    pub fn update(&mut self, blocks: &[Block], delta_time: f32, input: &PlayerInput) {
        const GRAVITY: f32 = -32.174; // Gravidade do Minecraft
        const JUMP_SPEED: f32 = 8.0;  // Velocidade de pulo

        // Atualizar direção da câmera (front)
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        self.front = vec3(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        ).normalize();

        // Resetar velocidade horizontal
        self.velocity.x = 0.0;
        self.velocity.z = 0.0;

        // Calcular direção de movimento
        let mut move_dir = Vec3::ZERO;
        if input.forward { move_dir += vec3(self.front.x, 0.0, self.front.z).normalize(); }
        if input.backward { move_dir -= vec3(self.front.x, 0.0, self.front.z).normalize(); }
        if input.left { move_dir -= vec3(-self.front.z, 0.0, self.front.x).normalize(); }
        if input.right { move_dir += vec3(-self.front.z, 0.0, self.front.x).normalize(); }

        // Normalizar direção
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        // Aplicar velocidade horizontal
        self.velocity.x = move_dir.x * self.speed;
        self.velocity.z = move_dir.z * self.speed;

        // Aplicar pulo
        if input.jump && self.on_ground {
            self.velocity.y = JUMP_SPEED;
            self.on_ground = false;
        }

        // Aplicar gravidade
        self.velocity.y += GRAVITY * delta_time;

        // Atualizar posição
        let mut new_position = self.position;
        new_position.x += self.velocity.x * delta_time;
        new_position.y += self.velocity.y * delta_time;
        new_position.z += self.velocity.z * delta_time;

        // Resolver colisões
        self.handle_collisions(blocks, &mut new_position);

        // Atualizar estado
        self.on_ground = self.is_on_ground(blocks);
        self.position = new_position;

        // Debug
        println!("Input: {:?}", input);
        println!("Front: {:?}", self.front);
        println!("Velocity: {:?}", self.velocity);
        println!("Position: {:?}", self.position);
    }

    fn handle_collisions(&mut self, blocks: &[Block], new_pos: &mut Vec3) {
        let half_size = self.size * 0.5;
        let old_pos = self.position;

        // Resolver colisão por eixo, ajustando a posição diretamente
        // X
        if self.velocity.x != 0.0 {
            let mut aabb_min = vec3(new_pos.x - half_size.x, old_pos.y - half_size.y, old_pos.z - half_size.z);
            let mut aabb_max = vec3(new_pos.x + half_size.x, old_pos.y + half_size.y, old_pos.z + half_size.z);

            for block in blocks.iter().filter(|b| b.id != "minecraft:water") {
                let block_min = vec3(block.x as f32, block.y as f32, block.z as f32);
                let block_max = block_min + Vec3::ONE;

                if Self::aabb_intersects(aabb_min, aabb_max, block_min, block_max) {
                    if self.velocity.x > 0.0 {
                        new_pos.x = block_min.x - half_size.x - 0.001; // Pequeno offset pra evitar sobreposição
                        self.velocity.x = 0.0;
                    } else if self.velocity.x < 0.0 {
                        new_pos.x = block_max.x + half_size.x + 0.001;
                        self.velocity.x = 0.0;
                    }
                }
            }
        }

        // Y
        if self.velocity.y != 0.0 {
            let mut aabb_min = vec3(new_pos.x - half_size.x, new_pos.y - half_size.y, old_pos.z - half_size.z);
            let mut aabb_max = vec3(new_pos.x + half_size.x, new_pos.y + half_size.y, old_pos.z + half_size.z);

            for block in blocks.iter().filter(|b| b.id != "minecraft:water") {
                let block_min = vec3(block.x as f32, block.y as f32, block.z as f32);
                let block_max = block_min + Vec3::ONE;

                if Self::aabb_intersects(aabb_min, aabb_max, block_min, block_max) {
                    if self.velocity.y > 0.0 {
                        new_pos.y = block_min.y - half_size.y - 0.001;
                        self.velocity.y = 0.0;
                    } else if self.velocity.y < 0.0 {
                        new_pos.y = block_max.y + half_size.y + 0.001;
                        self.velocity.y = 0.0;
                        self.on_ground = true;
                    }
                }
            }
        }

        // Z
        if self.velocity.z != 0.0 {
            let mut aabb_min = vec3(new_pos.x - half_size.x, new_pos.y - half_size.y, new_pos.z - half_size.z);
            let mut aabb_max = vec3(new_pos.x + half_size.x, new_pos.y + half_size.y, new_pos.z + half_size.z);

            for block in blocks.iter().filter(|b| b.id != "minecraft:water") {
                let block_min = vec3(block.x as f32, block.y as f32, block.z as f32);
                let block_max = block_min + Vec3::ONE;

                if Self::aabb_intersects(aabb_min, aabb_max, block_min, block_max) {
                    if self.velocity.z > 0.0 {
                        new_pos.z = block_min.z - half_size.z - 0.001;
                        self.velocity.z = 0.0;
                    } else if self.velocity.z < 0.0 {
                        new_pos.z = block_max.z + half_size.z + 0.001;
                        self.velocity.z = 0.0;
                    }
                }
            }
        }
    }

    fn aabb_intersects(min1: Vec3, max1: Vec3, min2: Vec3, max2: Vec3) -> bool {
        min1.x < max2.x && max1.x > min2.x &&
        min1.y < max2.y && max1.y > min2.y &&
        min1.z < max2.z && max1.z > min2.z
    }

    fn is_on_ground(&self, blocks: &[Block]) -> bool {
        let half_size = self.size * 0.5;
        let feet_pos = self.position - vec3(0.0, half_size.y + 0.001, 0.0);
        let player_min = feet_pos - half_size;
        let player_max = feet_pos + half_size;

        blocks.iter()
            .filter(|b| b.id != "minecraft:water")
            .any(|b| {
                let block_min = vec3(b.x as f32, b.y as f32, b.z as f32);
                let block_max = block_min + Vec3::ONE;
                Self::aabb_intersects(player_min, player_max, block_min, block_max)
            })
    }
}

#[derive(Default, Debug)]
pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

// ... (load_texture permanece igual)

#[inline]
pub fn load_texture(path: &str) -> GLuint {
    let mut texture_id = 0;
    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

        if let Ok(img) = image::open(path) {
            let data = img.flipv().to_rgba8();
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
        } else {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                1,
                1,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                [255, 0, 255, 255].as_ptr() as *const _,
            );
        }
    }
    texture_id
}