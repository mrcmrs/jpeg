// cargo bench --bench my_benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpeg::decode_binary;
use std::fs;

pub fn benchmark_decode(c: &mut Criterion) {
    let img_path = "img/maps.jpg";

    let data = fs::read(img_path).expect("Failed to read JPEG file");

    c.bench_function("decode_jpeg", |b| {
        b.iter(|| {
            decode_binary(black_box(&data));
        })
    });
}

criterion_group!(benches, benchmark_decode);
criterion_main!(benches);


