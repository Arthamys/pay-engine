use parser::Parser;
use pay_engine::*;
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

pub fn run_engine(filepath: Option<String>) -> Result<()> {
    let mut transactions = get_transaction_stream(filepath)?;
    let (wallets, _) = engine::run(&mut transactions);

    wallets.print_balances().map_err(|e| {
        eprintln!("Could not serialize wallet balances ({})", e);
        error::Error::SerializeError
    })
}

fn get_transaction_stream(
    filepath: Option<String>,
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
    return Ok(Box::new(RandomTransactions::new().take(10_000_000)));
}
