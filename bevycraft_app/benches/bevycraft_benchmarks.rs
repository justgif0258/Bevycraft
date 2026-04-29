use std::hint::black_box;
use criterion::{Criterion, criterion_group, criterion_main};
use noise::NoiseFn;
use simdnoise::NoiseBuilder;

const SEED: i32 = 123;

const CHUNK_SIZE: usize = 1024 * 16;

pub fn noise(c: &mut Criterion) {
    let mut sample: Box<[f32; CHUNK_SIZE]> = Box::new([0.0; CHUNK_SIZE]);

    c.bench_function("fill recursive", |b| {
        b.iter(|| fill_recursive(black_box(&mut sample)))
    });

    c.bench_function("fill simd", |b| {
        b.iter(|| fill_simd(black_box(&mut sample)))
    });
}

fn fill_recursive(arr: &mut [f32; CHUNK_SIZE]) {
    let perlin = noise::Perlin::new(SEED as u32);

    arr.iter_mut()
        .enumerate()
        .for_each(|(i, val)| {
            *val = perlin.get([i as f64]) as f32;
        })
}

fn fill_simd(arr: &mut [f32; CHUNK_SIZE]) {
    let (computed, _, _) = NoiseBuilder::fbm_1d(CHUNK_SIZE)
        .with_seed(SEED)
        .generate();

    arr.iter_mut()
        .enumerate()
        .for_each(|(i, val)| {
            *val = computed[i];
        })
}

criterion_group!(benches, noise);
criterion_main!(benches);
