use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

fn write_png(path: &Path, pixels: &[u8], width: u32, height: u32) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixels)?;
    Ok(())
}

#[inline]
fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn grad(hash: u8, x: f32, y: f32) -> f32 {
    let vectors = [(0.0, 1.0), (0.0, -1.0), (1.0, 0.0), (-1.0, 0.0)];
    let grad_factor = vectors[(hash & 3) as usize];
    grad_factor.0 * x + grad_factor.1 * y
}

fn generate_permutation(seed: u64) -> Vec<u8> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut permutation: Vec<u8> = (0..=255).collect();
    permutation.shuffle(&mut rng);

    permutation.push(permutation[0]);

    permutation
}

fn perlin_noise(x: f32, y: f32, permutation: &[u8]) -> f32 {
    let xi = (x.floor() as i32 & 255) as usize;
    let yi = (y.floor() as i32 & 255) as usize;

    let xx = x.fract();
    let yy = y.fract();

    let u = fade(xx);
    let v = fade(yy);

    let a = (permutation[xi] as usize + yi) & 255;
    let b = (permutation[xi + 1] as usize + yi) & 255;

    lerp(
        v,
        lerp(
            u,
            grad(permutation[a], xx, yy),
            grad(permutation[b], xx - 1.0, yy),
        ),
        lerp(
            u,
            grad(permutation[a + 1], xx, yy - 1.0),
            grad(permutation[b + 1], xx - 1.0, yy - 1.0),
        ),
    )
}

fn fractal_brownian_motion(x: f32, y: f32, n_octaves: usize, permutation: &[u8]) -> f32 {
    let mut result = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 0.005;

    for _ in 0..n_octaves {
        let n = amplitude * perlin_noise(x * frequency, y * frequency, permutation);
        result += n;

        amplitude *= 0.5;
        frequency *= 2.0;
    }

    result
}

fn generate_noise(width: u32, height: u32, n_octaves: usize, seed: u64) {
    let image_path = Path::new(r"outputs/test.png");
    let permutation = generate_permutation(seed);

    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let noise = fractal_brownian_motion(x as f32, y as f32, n_octaves, &permutation);
            let intensity = (255.0 * (noise + 1.0) / 2.0) as u8;

            match intensity {
                0..=100 => pixels.extend([67, intensity + 65, intensity + 155, 255]),
                101..=140 => pixels.extend([intensity + 60, intensity, 44, 255]),
                141..=210 => pixels.extend([92, intensity + 45, 36, 255]),
                _ => pixels.extend([244, 249, 255, 255]),
            }
        }
    }

    write_png(image_path, &pixels, width, height).unwrap();
}

fn main() {
    let seed = rand::random();
    println!("{seed}");
    generate_noise(1024, 1024, 8, seed);
}
