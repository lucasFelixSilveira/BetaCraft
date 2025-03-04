use noise::{Fbm, MultiFractal, NoiseFn, Perlin, Seedable};
use rand::{Rng, thread_rng};
use crate::game::blocks::Block;

const SEA_LEVEL: i32 = 62;
const TERRAIN_SCALE: f64 = 300.0;
const MOUNTAIN_SCALE: f64 = 1000.0;
const TREE_CHANCE: f64 = 0.015;

#[derive(Debug, Clone, PartialEq)]
enum Biome {
    Desert,
    Plains,
    Mountains,
    Ocean,
}

// Retorna os blocos e a coordenada de spawn (x, y, z)
pub fn assembly(size_x: i32, size_z: i32) -> (Vec<Block>, (i32, i32, i32)) {
    let mut blocks = Vec::with_capacity((size_x * size_z * 128) as usize);
    let rng = thread_rng();
    
    let (terrain_noise, mountain_noise, biome_noise) = init_noise_generators(rng);
    let noise_maps = precompute_noise(size_x, size_z, &terrain_noise, &mountain_noise, &biome_noise);
    
    // Mapa temporário para rastrear alturas da superfície
    let mut surface_map = vec![None; (size_x * size_z) as usize];
    
    for x in 0..size_x {
        for z in 0..size_z {
            let idx = (x * size_z + z) as usize;
            let (base_height, mountain_height, biome_value) = noise_maps[idx];
            
            let (height, biome) = determine_biome_and_height(base_height, mountain_height, biome_value);
            generate_column(x, z, height, biome.clone(), &mut blocks);
            
            // Registra a altura da superfície para spawn
            if matches!(biome, Biome::Plains | Biome::Desert) {
                surface_map[idx] = Some(height);
            }
        }
    }
    
    generate_features(size_x, size_z, &mut blocks);
    
    // Escolhe uma coordenada de spawn segura
    let spawn_point = find_spawn_point(&surface_map, size_x, size_z, &blocks);
    
    (blocks, spawn_point)
}

fn init_noise_generators(mut rng: impl Rng) -> (Fbm<Perlin>, Fbm<Perlin>, Perlin) {
    (
        Fbm::<Perlin>::new(rng.gen())
            .set_octaves(3)
            .set_frequency(1.0 / TERRAIN_SCALE)
            .set_persistence(0.4),
        Fbm::<Perlin>::new(rng.gen())
            .set_octaves(4)
            .set_frequency(1.0 / MOUNTAIN_SCALE)
            .set_persistence(0.6),
        Perlin::new(rng.gen())
    )
}

fn precompute_noise(size_x: i32, size_z: i32, 
    terrain: &Fbm<Perlin>, 
    mountain: &Fbm<Perlin>, 
    biome: &Perlin) -> Vec<(i32, i32, f64)> {
    
    let mut noise_values = Vec::with_capacity((size_x * size_z) as usize);
    
    for x in 0..size_x {
        let nx = x as f64;
        for z in 0..size_z {
            let nz = z as f64;
            noise_values.push((
                (terrain.get([nx, nz]) * 32.0 + SEA_LEVEL as f64) as i32,
                (mountain.get([nx, nz]).abs() * 80.0) as i32,
                biome.get([nx, nz])
            ));
        }
    }
    noise_values
}

#[inline]
fn determine_biome_and_height(base: i32, mountain: i32, biome_value: f64) -> (i32, Biome) {
    match biome_value {
        v if v < -0.4 => (base + 5, Biome::Desert),
        v if v < 0.1 => (base + 10, Biome::Plains),
        v if v < 0.6 => (base + mountain, Biome::Mountains),
        _ => (SEA_LEVEL - 10, Biome::Ocean),
    }
}

#[inline]
fn generate_column(x: i32, z: i32, height: i32, biome: Biome, blocks: &mut Vec<Block>) {
    let (surface, subsurface) = match biome {
        Biome::Desert => ("minecraft:sand", "minecraft:sand"),
        Biome::Plains => ("minecraft:grass_block", "minecraft:dirt"),
        Biome::Mountains => ("minecraft:stone", "minecraft:stone"),
        Biome::Ocean => ("minecraft:sand", "minecraft:dirt"),
    };

    for y in 1..=height {
        let block_type = if y == height {
            surface
        } else if y > height - 3 {
            subsurface
        } else {
            "minecraft:stone"
        };
        blocks.push(Block::new(block_type, x, y, z));
    }

    if biome == Biome::Ocean && height < SEA_LEVEL {
        blocks.extend((height + 1..=SEA_LEVEL).map(|y| 
            Block::new("minecraft:water", x, y, z)
        ));
    }
    
    blocks.push(Block::new("minecraft:bedrock", x, 0, z));
}

fn generate_features(size_x: i32, size_z: i32, blocks: &mut Vec<Block>) {
    let mut rng = thread_rng();
    let mut surface_map = vec![None; (size_x * size_z) as usize];

    for block in blocks.iter() {
        if block.id == "minecraft:grass_block" || block.id == "minecraft:sand" || block.id == "minecraft:stone" {
            let idx = (block.x * size_z + block.z) as usize;
            surface_map[idx] = Some(block.y);
        }
    }

    for x in 0..size_x {
        for z in 0..size_z {
            if rng.gen_bool(TREE_CHANCE) {
                if let Some(surface_y) = surface_map[(x * size_z + z) as usize] {
                    generate_tree(x, z, surface_y, blocks, &mut rng);
                }
            }
        }
    }
}

#[inline]
fn generate_tree(x: i32, z: i32, surface_y: i32, blocks: &mut Vec<Block>, rng: &mut impl Rng) {
    let trunk_height = rng.gen_range(4..7);
    blocks.extend((surface_y + 1..=surface_y + trunk_height)
        .map(|y| Block::new("minecraft:oak_log", x, y, z)));

    let crown_y = surface_y + trunk_height;
    for dx in -2i32..=2i32 {
        for dz in -2i32..=2i32 {
            for dy in -1i32..=1i32 {
                if (dx.abs() != 2 || dz.abs() != 2) && rng.gen_bool(0.7) {
                    blocks.push(Block::new("minecraft:oak_leaves", 
                        x + dx, 
                        crown_y + dy, 
                        z + dz));
                }
            }
        }
    }
}

// Função para encontrar um ponto de spawn seguro
fn find_spawn_point(surface_map: &[Option<i32>], size_x: i32, size_z: i32, blocks: &[Block]) -> (i32, i32, i32) {
    let mut rng = thread_rng();
    
    // Tenta encontrar um ponto em Plains ou Desert com espaço vazio acima
    for _ in 0..100 { // Limite de tentativas para evitar loop infinito
        let x = rng.gen_range(0..size_x);
        let z = rng.gen_range(0..size_z);
        let idx = (x * size_z + z) as usize;
        
        if let Some(surface_height) = surface_map[idx] {
            // Verifica se o espaço acima da superfície está 100% vazio (2 blocos de altura para o jogador)
            let spawn_y = surface_height + 1; // Um bloco acima da superfície
            let is_space_clear = blocks.iter()
                .filter(|b| b.x == x && b.z == z && b.y >= spawn_y && b.y <= spawn_y + 1)
                .all(|b| b.id == "minecraft:water" || b.id == "minecraft:air"); // Considera apenas blocos sólidos como obstáculos

            if is_space_clear {
                // Spawn dois blocos acima da superfície para garantir que o jogador esteja acima da última camada
                return (x, spawn_y + 1, z); // +1 para ficar acima do chão, totalizando 2 blocos acima da superfície
            }
        }
    }
    
    // Fallback: centro do mundo, altura média, garantindo espaço vazio
    let fallback_x = size_x / 2;
    let fallback_z = size_z / 2;
    let idx = (fallback_x * size_z + fallback_z) as usize;
    let surface_height = surface_map[idx].unwrap_or(SEA_LEVEL);
    (fallback_x, surface_height + 2, fallback_z) // 2 blocos acima como fallback
}