use std::hint::black_box;
use criterion::{Criterion, criterion_group, criterion_main};

const SEED: i32 = 123;

const CHUNK_SIZE: usize = 1024 * 16;

pub fn noise(c: &mut Criterion) {
    unimplemented!()
}

criterion_group!(benches, noise);
criterion_main!(benches);
