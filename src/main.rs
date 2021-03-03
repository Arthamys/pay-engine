use parser::Parser;
use pay_engine::*;
use simple_logger::SimpleLogger;

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
    let parser = match Parser::new(&filepath) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            return Ok(());
        }
    };

    let (wallets, _) = engine::run(&mut parser.into_iter());

    wallets.print_balances().map_err(|e| {
        eprintln!("Could not serialize wallet balances ({})", e);
        error::Error::SerializeError
    })
}
