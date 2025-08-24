#![allow(unused_variables, unused_imports, dead_code)]
use std::hint::black_box;

use cedar::core::manifest::{EXAMPLE_MANIFEST, Manifest};
use criterion::{Criterion, criterion_group, criterion_main};

mod misc_impl;

fn core_manifest_b(c: &mut Criterion) {
    c.bench_function("manifest_parse_std", |b| {
        b.iter(|| {
            let manifest = Manifest::parse(black_box(EXAMPLE_MANIFEST)).unwrap();
            black_box(manifest);
        })
    });
}

criterion_group!(benches, core_manifest_b);
criterion_main!(benches);
