#![allow(unused_variables, unused_imports, dead_code)]
use std::hint::black_box;

use cedar::core::{
    dag::DependGraph,
    manifest::{EXAMPLE_MANIFEST, Manifest},
};
use criterion::{Criterion, criterion_group, criterion_main};

fn core_manifest_b(c: &mut Criterion) {
    c.bench_function("manifest_parse_std", |b| {
        b.iter(|| {
            let manifest = Manifest::parse(black_box(EXAMPLE_MANIFEST)).unwrap();
            black_box(manifest);
        })
    });
}

fn core_dag_b(c: &mut Criterion) {
    c.bench_function("dag_init", |b| {
        b.iter(|| {
            let dag = DependGraph::build(black_box("/home/june/archive/repo/gcc/"));
            black_box(dag);
        })
    });
}

criterion_group!(benches, core_manifest_b, core_dag_b);
criterion_main!(benches);
