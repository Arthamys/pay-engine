use pay_engine::{engine, transaction::utils::RandomTransactions};
use std::iter::Iterator;

#[test]
#[ignore]
fn exec_100_million_tx() {
    let tx_gen = RandomTransactions::new();
    engine::Engine::new().run(
        &mut tx_gen
            .into_iter()
            .take(/*u32::max_value()*/ 100_000_000 as usize),
    );
}

#[test]
fn exec_1_million_tx() {
    let tx_gen = RandomTransactions::new();
    engine::Engine::new().run(
        &mut tx_gen
            .into_iter()
            .take(/*u32::max_value()*/ 1_000_000 as usize),
    );
}
