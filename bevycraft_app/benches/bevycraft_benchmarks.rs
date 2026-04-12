use criterion::{Criterion, criterion_group, criterion_main};

pub fn array_access_mark(c: &mut Criterion) {
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
}

criterion_group!(benches, array_access_mark);
criterion_main!(benches);
