use super::dna::*;
use super::map3::*;
use super::map3base::*;
use super::math::*;
use super::voronoi::*;
use super::*;

pub fn genmap3(complexity: f32, dna: &mut Dna) -> Box<dyn Texture> {
    let basis_weight = { 0.20 };
    let unary_weight = if complexity > 5.0 { 0.30 } else { 0.0 };
    let binary_weight = if complexity > 8.0 { 0.25 } else { 0.0 };

    let x = dna.get_f32_in(0.0, basis_weight + unary_weight + binary_weight);

    if x < basis_weight {
        let frequency = xerp(1.5, 32.0, dna.get_f32());
        let texture: Box<dyn Texture> = match dna.get_u32_in(0, 1) {
            0 => vnoise(0, frequency, tile_none()),
            _ => {
                let pattern_x = dna.get_u32_in(0, 12);
                let pattern_y = dna.get_u32_in(0, 12);
                let pattern_z = dna.get_u32_in(0, 12);
                voronoi(
                    0,
                    frequency,
                    tile_none(),
                    pattern_x as usize,
                    pattern_y as usize,
                    pattern_z as usize,
                )
            }
        };
        texture
    } else if x < basis_weight + unary_weight {
        let child_complexity = complexity * 0.5 - 1.0;
        let child = dna.call(|dna| genmap3(child_complexity, dna));
        let unary_node = match dna.get_u32_in(0, 4) {
            0 => {
                let amount = dna.get_f32_in(2.0, 10.0);
                saturate(amount, child)
            }
            1 => {
                let levels = dna.get_f32_in(2.0, 8.0);
                let sharpness: f32 = dna.get_f32();
                posterize(levels, sharpness, child)
            }
            2 => {
                let amount = dna.get_f32_in(2.0, 10.0);
                overdrive(amount, child)
            }
            3 => {
                let amount = dna.get_f32_in(2.0, 10.0);
                vreflect(amount, child)
            }
            _ => {
                let amount = dna.get_f32_in(2.0, 10.0);
                let x_offset = dna.get_f32_in(-1.0, 1.0);
                let y_offset = dna.get_f32_in(-1.0, 1.0);
                let z_offset = dna.get_f32_in(-1.0, 1.0);
                reflect(amount, vec3a(x_offset, y_offset, z_offset), child)
            }
        };
        unary_node
    } else {
        let child_complexity = complexity * 0.5 - 1.0;
        let child_a = dna.call(|dna| genmap3(child_complexity, dna));
        let child_b = dna.call(|dna| genmap3(child_complexity, dna));
        let binary_node = match dna.get_u32_in(0, 2) {
            0 => {
                let amount = dna.get_f32_in(3.0, 10.0);
                rotate(amount, child_a, child_b)
            }
            1 => {
                let amount = dna.get_f32_in(1.0, 10.0);
                softmix3(amount, child_a, child_b)
            }
            _ => {
                let amount = dna.get_f32_in(0.05, 0.25);
                displace(amount, child_a, child_b)
            }
        };
        binary_node
    }
}
