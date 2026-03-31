use bevycraft_core::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use rand::random_range;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hint::black_box;
use std::time::Instant;

pub fn array_access_mark(c: &mut Criterion) {
    let pool = VirtualizedPool::<u8>::new(4096, 1024 * 8).unwrap();

    /*
    let namespaces = [
        "bevycraft",
        "minecraft",
        "my_mod",
        "example_mod",
        "some_mod",
    ];

    let paths = [
        "stone",
        "cobblestone",
        "dirt",
        "grass_block",
        "bedrock",
        "water",
        "sand",
        "sandstone",
        "gravel",
        "diamond_block",
        "obsidian",
        "bed",
    ];

    c.bench_function("RegistrationId Create/Intern", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            (0..iters).into_iter().for_each(|_| {
                let n = random_range(0..5usize);
                let p = random_range(0..12usize);

                let id = RegistrationId::with_custom_namespace(namespaces[n], paths[p]);

                black_box(id);
            });
            start.elapsed()
        })
    });

    c.bench_function("RegistrationId Create/Intern (Multi-threaded)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            (0..iters).into_par_iter().for_each(|_| {
                let n = random_range(0..5usize);
                let p = random_range(0..12usize);

                let id = RegistrationId::with_custom_namespace(namespaces[n], paths[p]);

                black_box(id);
            });
            start.elapsed()
        })
    });
    */

    c.bench_function("Virtualized Pool Commit/Decommit (Multi-threaded)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            (0..iters).into_par_iter().for_each(|i| {
                let page = pool.commit().unwrap();

                page.write((i & 0b111111111111) as usize, i as u8);

                black_box(page);
            });
            start.elapsed()
        })
    });

    c.bench_function("Box Create/Drop (Multi-threaded)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            (0..iters).into_par_iter().for_each(|i| {
                let mut box1 = Box::new([0u8; 4096]);

                unsafe {
                    box1.as_mut_ptr()
                        .add((i & 0b111111111111) as usize)
                        .write(i as u8);
                }

                black_box(box1);
            });
            start.elapsed()
        })
    });
}

criterion_group!(benches, array_access_mark);
criterion_main!(benches);
