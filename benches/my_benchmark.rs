use criterion::{criterion_group, criterion_main, Criterion};
use pay_engine::{engine, transaction::utils::RandomTransactions};
use std::iter::Iterator;
use std::time::Duration;

fn exec_1_million_tx() {
    let tx_gen = RandomTransactions::new();
    engine::run(
        &mut tx_gen
            .into_iter()
            .take(/*u32::max_value()*/ 1_000_000 as usize),
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    group
        .sample_size(100)
        .warm_up_time(Duration::from_secs(140))
        .measurement_time(Duration::from_secs(33));
    group.bench_function("tx 1mil", |b| b.iter(|| exec_1_million_tx()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
