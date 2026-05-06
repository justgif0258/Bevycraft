use bevycraft_core::prelude::PatternContainer;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

const CHUNK_SIZE: usize = 4096;

pub fn compression(c: &mut Criterion) {
    let mut container: PatternContainer<u32, CHUNK_SIZE> =
        PatternContainer::with_bit_capacity(0, 12);

    let mut rand = fastrand::Rng::new();

    c.bench_function("PatternContainer/RandomWrite", |b| {
        b.iter(|| {
            let index = rand.usize(..CHUNK_SIZE);
            let value = rand.u32(..512);

            container.set(index, value);
        })
    });

    c.bench_function("PatternContainer/RandomRead", |b| {
        b.iter(|| {
            let index = rand.usize(..CHUNK_SIZE);

            let value = container.get(index);
            black_box(value);
        })
    });

    println!(
        "Total entries: {}\nCurrent bit capacity: {}",
        container.active_entries(),
        container.bit_capacity()
    );

    let result = container.try_compress();
    println!("Compression result:");
    println!("  - Success: {}", result);
    println!("  - Entries: {}", container.active_entries());
    println!("  - Bit capacity: {}", container.bit_capacity());
}

criterion_group!(benches, compression);
criterion_main!(benches);
