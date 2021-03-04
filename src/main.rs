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
    let _prog_name = args
        .next()
        .expect("USAGE: cargo run #Runs with random number of tx");

    run_engine(args.next()).map_err(|e| {
        eprintln!("Could not run engine ({})", e);
        error::Error::DeserializeError
    })?;
    Ok(())
}

/// Run the payment engine on the transactions provided,  or randomly generated
pub fn run_engine(filepath: Option<String>) -> Result<()> {
    let mut transactions = get_transaction_stream(&filepath)?;
    let total_transactions = transactions.size_hint().1.unwrap_or(1);
    let before = Instant::now();

    let mut engine = engine::Engine::new();
    let wallets = engine.run(&mut transactions);

    let runtime = before.elapsed().as_secs_f32();

    let res = if filepath.is_some() {
        wallets.print_balances().map_err(|e| {
            eprintln!("Could not serialize wallet balances ({})", e);
            error::Error::SerializeError
        })
    } else {
        Ok(())
    };

    log::error!(
        "Executed {} successfull transactions ({}%) out of {} in {:.04}s",
        engine.valid_transactions(),
        ((100 * engine.valid_transactions()) / total_transactions),
        total_transactions,
        runtime
    );
    res
}

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
