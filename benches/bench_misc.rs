#![allow(unused_variables, unused_imports, dead_code)]
use std::hint::black_box;

use cedar::core::hash::{fx_content_hash, fx_content_hash_words};
use criterion::{Criterion, criterion_group, criterion_main};

fn core_manifest(c: &mut Criterion) {}

fn core_hash(c: &mut Criterion) {
    let paths = [
        "./tests/loose/random_0016kb_t",
        "./tests/loose/random_0064kb_t",
        "./tests/loose/random_0256kb_t",
        "./tests/loose/random_1024kb_t",
        "./tests/loose/random_4096kb_t",
    ];
    c.bench_function("fx_byte_16kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash(black_box(paths[0]));
            black_box(hash);
        })
    });
    c.bench_function("fx_word_16kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash_words(black_box(paths[0]));
            black_box(hash);
        })
    });
    c.bench_function("fx_byte_64kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash(black_box(paths[1]));
            black_box(hash);
        })
    });
    c.bench_function("fx_word_64kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash_words(black_box(paths[1]));
            black_box(hash);
        })
    });
    c.bench_function("fx_byte_256kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash(black_box(paths[2]));
            black_box(hash);
        })
    });
    c.bench_function("fx_word_256kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash_words(black_box(paths[2]));
            black_box(hash);
        })
    });
    c.bench_function("fx_byte_1024kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash(black_box(paths[3]));
            black_box(hash);
        })
    });
    c.bench_function("fx_word_1024kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash_words(black_box(paths[3]));
            black_box(hash);
        })
    });
    c.bench_function("fx_byte_4096kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash(black_box(paths[4]));
            black_box(hash);
        })
    });
    c.bench_function("fx_word_4096kb", |b| {
        b.iter(|| {
            let hash = fx_content_hash_words(black_box(paths[4]));
            black_box(hash);
        })
    });
}

criterion_group!(benches, core_hash, core_manifest);
criterion_main!(benches);
