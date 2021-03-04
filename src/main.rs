use parser::Parser;
use pay_engine::*;
use std::time::Instant;
use transaction::{utils::RandomTransactions, Transaction};

use simple_logger::SimpleLogger;

fn main() -> Result<()> {
    SimpleLogger::new()
        // change here to enable logs
        .with_level(log::LevelFilter::Off)
        .init()
        .unwrap();

    // skip program name
    let mut args = std::env::args();
    let _prog_name = args.next().expect("USAGE: cargo run");

    run_engine(args.next()).map_err(|e| {
        eprintln!("Could not run engine ({})", e);
        error::Error::DeserializeError
    })?;
    Ok(())
}

/// If a file is given, open it and parse it as CSV.
/// Otherwise, generate random transactions
fn run_engine(filepath: Option<String>) -> Result<()> {
    let gen_random_tx = filepath.is_none();

    let mut transactions = get_transaction_stream(&filepath)?;
    let total_transactions = transactions.size_hint().1.unwrap_or(1);
    let before = Instant::now();
    let (wallets, tx_log) = engine::run(&mut transactions);
    let runtime = before.elapsed().as_secs_f32();

    if !gen_random_tx {
        wallets.print_balances().map_err(|e| {
            eprintln!("Could not serialize wallet balances ({})", e);
            error::Error::SerializeError
        })?;
    } else {
        println!(
            "Executed {} successfull transactions ({}%) out of {} in {:.04}s",
            tx_log.len(),
            ((100 * tx_log.len()) / total_transactions),
            total_transactions,
            runtime
        );
    }
    Ok(())
}

/// get a trait object for our transactions
fn get_transaction_stream(
    filepath: &Option<String>,
) -> Result<Box<dyn Iterator<Item = Transaction>>> {
    if let Some(file) = filepath {
        match Parser::new(&file) {
            Ok(p) => return Ok(Box::new(p)),
            Err(err) => {
                eprintln!("{}", err);
                return Err(err);
            }
        }
    }
    return Ok(Box::new(RandomTransactions::new().take(1_000_000)));
}
