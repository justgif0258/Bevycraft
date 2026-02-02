use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use bevycraft_core::prelude::*;

pub fn array_access_mark(c: &mut Criterion) {
    let mut arr = PackedArrayU32::new(4096);

    c.bench_function("PackedIndexArray write mark", |b| b.iter(|| arr.set(562, black_box(5))));
    c.bench_function("PackedIndexArray access mark", |b| b.iter(|| black_box(arr.get(1560))));

    //c.bench_function("PackedIndexArray access mark 4096 indices", |b| b.iter(|| read_4096(&arr)));
    //c.bench_function("PackedIndexArray write mark 4096 indices", |b| b.iter(|| write_4096(&mut arr)));

    //c.bench_function("PackedIndexArray resize bits mark", |b| b.iter(|| { arr.resize_bits(1); arr = PackedArrayU32::with_bit_length(4096, 1); }));

    println!("{:?}", arr);
    println!("{}", arr.get(2451));
}

fn write_4096(arr: &mut PackedArrayU32) {
    for i in 0..4096 {
        arr.set(i, black_box(i as u32));
    }
}

fn read_4096(arr: &PackedArrayU32) {
    for i in 0..4096 {
        black_box(arr.get(i));
    }
}

criterion_group!(benches, array_access_mark);
criterion_main!(benches);