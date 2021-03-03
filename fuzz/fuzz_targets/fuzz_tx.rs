#![no_main]
use libfuzzer_sys::fuzz_target;
use pay_engine::{engine::run, transaction::Transaction};

fuzz_target!(|data: Vec<Transaction>| {
    run(&mut data.into_iter());
});
