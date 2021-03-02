mod client;
mod engine;
mod error;
mod parser;
mod transaction;

use client::ClientWallets;
pub use error::Result;
use parser::Parser;
use simple_logger::SimpleLogger;
use transaction::{Transaction, TransactionLog};

fn main() -> Result<()> {
    SimpleLogger::new()
        // change here to enable logs
        .with_level(log::LevelFilter::Off)
        .init()
        .unwrap();

    // skip program name
    let mut args = std::env::args();
    let prog_name = args.next().expect("USAGE: cargo run -- input.csv");

    if let Some(filepath) = args.next() {
        run_engine(filepath).map_err(|e| {
            eprintln!("Could not run engine ({})", e);
            error::Error::DeserializeError
        })?;
    } else {
        println!("USAGE: {} intput.csv > output.csv", prog_name);
    }
    Ok(())
}

pub fn run_engine(filepath: String) -> Result<()> {
    let mut parser = match Parser::new(&filepath) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return Ok(());
        }
    };

    let mut transactions = TransactionLog::new();
    let mut wallets = ClientWallets::new();

    // We simply parse the file line by line, and use the engine to apply each
    // transaction
    while let Some(t) = parser.next().map_err(|_e| error::Error::DeserializeError)? {
        // find client in the list we have, if it does not exits, create it
        let mut client = wallets.get_or_create_mut(t.client);
        engine::execute_transaction(&t, &mut transactions, &mut client);
    }

    wallets.print_balances().map_err(|e| {
        eprintln!("Could not serialize wallet balances ({})", e);
        error::Error::SerializeError
    })
}
