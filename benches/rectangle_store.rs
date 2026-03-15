#![allow(unused)]

use std::hint::black_box;

use criterion::{
    BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use hui::{Rectangle, RectangleStore};

fn make_rect() -> Rectangle {
    Rectangle::builder()
        .mvp([[1.0, 0.0, 0.0, 0.0]; 4])
        .fill_color([1.0, 0.0, 0.0, 1.0])
        .border_color([0.0, 0.0, 0.0, 1.0])
        .corner_radii([4.0, 4.0, 4.0, 4.0])
        .shadow_color([0.0, 0.0, 0.0, 0.5])
        .clip_rect([0.0, 0.0, 1920.0, 1080.0])
        .half_size([50.0, 25.0])
        .border_size(1.0)
        .shadow_spread(0.0)
        .shadow_offset([0.0, 0.0])
        .shadow_blur(0.0)
        .build()
}

fn bench_single(c: &mut Criterion) {
    let rect = make_rect();
    let mut group = c.benchmark_group("single");

    group.bench_function("add_remove", |b| {
        let mut pool = RectangleStore::new();
        b.iter(|| {
            let id = pool.add(black_box(&rect));
            pool.remove(black_box(id))
        });
    });

    group.bench_function("build_after_add", |b| {
        b.iter(|| {
            let mut pool = RectangleStore::new();
            pool.add(black_box(&rect));
            black_box(pool.bytes());
        });
    });

    group.finish();
}

fn bench_scaled(c: &mut Criterion) {
    let rect = make_rect();
    let mut group = c.benchmark_group("scaled");

    for n in [10, 100, 500, 1024] {
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(
            BenchmarkId::new("full_rebuild", n),
            &n,
            |b, &n| {
                let mut pool = RectangleStore::new();
                let ids: Vec<_> = (0..n).map(|_| pool.add(&rect)).collect();

                b.iter(|| {
                    for &id in &ids {
                        if let Some(r) = pool.get_mut(black_box(id)) {
                            black_box(r);
                        }
                    }
                    black_box(pool.bytes());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("no_rebuild", n),
            &n,
            |b, &n| {
                let mut pool = RectangleStore::new();
                for _ in 0..n {
                    pool.add(&rect);
                }
                pool.bytes();

                b.iter(|| {
                    black_box(pool.bytes());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("partial_rebuild", n),
            &n,
            |b, &n| {
                let mut pool = RectangleStore::new();
                let ids: Vec<_> = (0..n).map(|_| pool.add(&rect)).collect();
                let dirty_count = (n / 10).max(1);

                b.iter(|| {
                    for &id in ids.iter().take(dirty_count) {
                        if let Some(r) = pool.get_mut(black_box(id)) {
                            black_box(r);
                        }
                    }
                    black_box(pool.bytes());
                });
            },
        );
    }

    group.finish();
}

fn bench_churn(c: &mut Criterion) {
    let rect = make_rect();
    let mut group = c.benchmark_group("churn");

    group.bench_function("add_build_remove_build", |b| {
        let mut pool = RectangleStore::new();
        for _ in 0..100 {
            pool.add(&rect);
        }

        b.iter(|| {
            let id = pool.add(black_box(&rect));
            black_box(pool.bytes());
            pool.remove(black_box(id));
            black_box(pool.bytes());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_single, bench_scaled, bench_churn);
criterion_main!(benches);
