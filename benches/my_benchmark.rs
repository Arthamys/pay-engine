use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pay_engine::{engine, transaction::utils::RandomTransactions};
use std::iter::Iterator;
use std::time::Duration;

fn exec_n_tx(n: usize) {
    let tx_gen = RandomTransactions::new();
    engine::run(&mut tx_gen.into_iter().take(n));
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tx_volume");
    group
        .sample_size(100)
        .warm_up_time(Duration::from_secs(33))
        .measurement_time(Duration::from_secs(66));
    for n in [1_000_usize, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("random_input", n), n, |b, n| {
            b.iter(|| exec_n_tx(*n))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
