// Copyright 2024-2025 Irreducible Inc.

use binius::bench::{prove, sha256_no_lookup_prepare, sha256_with_lookup_prepare, verify};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

fn sha256_no_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("sha256_no_lookup");
    group.sample_size(10);
    let allocator = bumpalo::Bump::new();

    group.bench_function("sha256_no_lookup_prove", |bench| {
        bench.iter_batched(
            || sha256_no_lookup_prepare(&allocator),
            |(constraint_system, args, witness, backend)| {
                prove(constraint_system, args, witness, backend);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("sha256_no_lookup_verify", |bench| {
        bench.iter_batched(
            || {
                let (constraint_system, args, witness, backend) =
                    sha256_no_lookup_prepare(&allocator);
                prove(constraint_system, args, witness, backend)
            },
            |(constraint_system, args, proof)| {
                verify(constraint_system, args, proof);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn sha256_with_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("sha256_with_lookup");
    group.sample_size(10);
    let allocator = bumpalo::Bump::new();

    group.bench_function("sha256_with_lookup_prove", |bench| {
        bench.iter_batched(
            || sha256_with_lookup_prepare(&allocator),
            |(constraint_system, args, witness, backend)| {
                prove(constraint_system, args, witness, backend);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("sha256_with_lookup_verify", |bench| {
        bench.iter_batched(
            || {
                let (constraint_system, args, witness, backend) =
                    sha256_with_lookup_prepare(&allocator);
                prove(constraint_system, args, witness, backend)
            },
            |(constraint_system, args, proof)| {
                verify(constraint_system, args, proof);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_main!(sha256);
criterion_group!(sha256, sha256_no_lookup, sha256_with_lookup);
