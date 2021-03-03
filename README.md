# Building and Running

To run the project, simply run `cargo run -- <input_file>` to invoke the payment engine on a given CSV file.

The program writes the balances of the clients encountered to the standard output.

## Testing

You can run
```bash
#make sure to be at the root of the repo
bash test.sh
```

to run a set of predefined tests agains the implementation.

It simply runs the program (invoking `cargo run`) and checks the output against known good outputs.

Some of the core functionality of for handling client's balances are also unit tested. (run `cargo t`)

## Fuzzing

Make sure you have `cargo-fuzz` installed (`cargo install cargo-fuzz`)

```bash
cargo +nightly fuzz run fuzz_tz -- -jobs=10 -rss_limit_mb=0 -len_control=0 -malloc_limit_mb=4096
```

I have not found any issues while fuzzing yet.

## Benchmarking

To run the benchmarks after making a modification, run

```bash
cargo bench
```

# Design

The idea is to have a _Parser_ check the validity of the input csv and output a
stream of _Operation_ s that can be executed by the engine.

The idea is to apply each _Operation_ to the _Engine_, which has a simple role of
running the business logic.
It acts on two data structures:

- a _ClientWallets_ structure that exposes a simple interface to request information from clients by their ID
  and to update their balances.
- a _TransactionLog_ that stores all valid processed _Transaction_. It exposes an interface to check wether a
  transaction already exists, wether there is a dispute underway for a certain transaction and so on.

If we run a transaction that is invalid, it does not get pushed to the TransactionLog.

# Improvements

- [ ] So far, these structures only store data in memory. This approach is not
      scalable, as the number of transactions grow. To make these structures more scalable, we could move the backing storage of the _TransactionLog_ to a database that is well tuned for writes, as we would most likely append data often, and query only for disputes.

- [ ] The Fixtures used for testing could be better. We could imagine a system 
      where we can create a simple list of transactions using macros. (transactions!(deposit!(1, 10.1), deposit!(2, 4.0), withdraw!(1, 44.134533))) -> Input Transactions to the engine
