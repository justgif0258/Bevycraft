use bevycraft_core::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;
use std::time::Instant;

pub fn array_access_mark(c: &mut Criterion) {
    let pool = VirtualizedPool::<u8>::new(4096, 1024 * 8).unwrap();

    c.bench_function("Virtualized Pool Commit/Decommit (Multi-threaded)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            (0..iters).into_par_iter().for_each(|i| {
                let page = pool.commit().unwrap();

                page.write((i & 0b111111111111) as usize, i as u8);
            });
            start.elapsed()
        })
    });

    c.bench_function("Box Create/Drop (Multi-threaded)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            (0..iters).into_par_iter().for_each(|_| {
                let box1 = Box::new(black_box([1u8; 4096]));

                black_box(box1);
            });
            start.elapsed()
        })
    });
}

criterion_group!(benches, array_access_mark);
criterion_main!(benches);
