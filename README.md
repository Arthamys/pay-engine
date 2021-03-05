# Pay Engine

A toy payment engine.

Given a list of transaction in a CSV file, apply the transactions to the wallets,
handle disputes on the transactions, and print the final balances of the clients.

## Transactions

Transactions will come in this form:
```text
type,       client, tx, amount
deposit,    5,      1,  42.2
deposit,    2,      2,  2.0
deposit,    3,      3,  2.0
withdrawal, 5,      5,  1.5
withdrawal, 2,      4,  3.0
dispute,    1,      1
chargeback, 1,      1
resolve,    1,      1
resolve,    1,      1
resolve,    1,      1
deposit,    2,      7,  3.8
```

## Wallets

A wallet tracks the balances of a client.

A Client has his funds split into two balances:
- `available` funds, which are ready to be used in transactions
- `held` funds, that are involved in disputed transactions (see below)

available|held|total
---------|----|-----
f64|f64|f64

## Types of operations

There are 5 kind of transactions:

### **Deposit**

Add funds to the wallet of the client that issued the transaction.
This should increase the client's available and total funds.

type|client|tx|amount
----|------|--|------
deposit|5|1|42.2

### **Withdrawal**

Remove funds from the wallet of the client that issued the transaction.
This should decrease the client's available and total funds.

type|client|tx|amount
----|------|--|------
withdrawal|5|5|2.2

If a client does not have sufficient funds to execute the withdrawal, discard
this transaction.

### **Dispute**

Dispute a transaction. When a transaction is disputed, it reverses
the operation, and puts the disputed funds into a holding balance in the client's
wallet.

type|client|tx|amount
----|------|--|------
dispute|5|1|

A transaction can only be disputed once. 

If the client issuing the transaction is not the same as the one that issued 
the linked `tx`, nothing happens.

Disputing a transaction that is already under dispute has no effect.

To resolve a dispute, the client has to issue a **Resolve** or **Chargeback**
transaction.

Notice that this does not have an **amount** field, but uses the `tx` field to 
refer to a transaction id directly.
If the transaction that is referenced does not exist, we can safely ignore the
transaction.

### **Resolve**

Resolve a dispute, releasing the client's held funds.
This should decrease the client's held balance, and increase his available balance.

type|client|tx|amount
----|------|--|------
resolve|5|1|

If the `tx` is not under dispute, nothing happens.
If the client issuing the transaction is not the same as the one that issued
the linked `tx`, nothing happens.

Notice that this does not have an **amount** field, but uses the `tx` field to 
refer to a transaction id directly.
If the transaction that is referenced does not exist, we can safely ignore the
transaction.

### **Chargeback**

This represents the client reversing a transaction. Held funds are now withdrawn
meaning we decrease the client's held and total funds.
If a chargeback occurs, the client's account should be locked.

type|client|tx|amount
----|------|--|------
chargeback|5|1|

If the `tx` is not under dispute, nothing happens.
If the client issuing the transaction is not the same as the one that issued
the linked `tx`, nothing happens.

Notice that this does not have an **amount** field, but uses the `tx` field to 
refer to a transaction id directly.
If the transaction that is referenced does not exist, we can safely ignore the
transaction.


# Building and Running

The project can be run against input CSV files if you have predefined scenarios
to run.

```bash
cargo run -q -- inputs/sample1.csv
# Expected output:
##################
# client,available,held,total,locked
# 1,1.5,0.0,1.5,false
# 2,2.0,0.0,2.0,false
```

It can also be invoked without a CSV file, and will generate random
transactions, and give some high level summary of what happened.

```bash
cargo run
# Expected output:
##################
# Executed 524695 successfull transactions (52%) out of 1000000 in 5.6524s
```

# Testing

There are two sources of tests in this projects. Simple "integration tests"
using a bash script to run the program on known inputs, and checks 
the output against known good outputs.

This is obviously very flimsy, but it is sufficient to serve as basic integration tests.

```bash
#make sure to be at the root of the repo
bash test.sh
```

Some of the core functionality of for handling client's balances are also unit tested.
Simply run

```bash
cargo test
```

## Fuzzing

I tried to use [fuzzing](https://en.wikipedia.org/wiki/Fuzzing) to generate random
but semi-valid transactions, but it turns out that fuzzing did not really do the
trick to generate long streams of close to valid inputs.

I used `cargo fuzz` to fuzz my program's input, which only tested the robustness
of the Parser I think.

If you want to try out the fuzzing module, Make sure you have `cargo-fuzz`
installed (`cargo install cargo-fuzz`) then run:

```bash
cargo +nightly fuzz run fuzz_tz -- -jobs=10 -rss_limit_mb=0 -len_control=0 -malloc_limit_mb=4096
```

I have not found any issues while fuzzing the program's input yet, but I bet my
method is not good. If you know how I could make fuzzing pertinent for this
project, please reach out.

## Benchmarking

Measure is key when trying to improve performance.

### Flamegraph

To have a good idea of where the hot zones of our code are, we can use the
`cargo flamegraph` tool (`cargo install flamegraph` if you do not have it installed).

```bash
cargo flamegraph --bin pay-engine && firefox flamegraph.svg
```

This runs the engine agaisnt 1 million randomly generated transactions, giving us
a small but workable sample to see what are the core functions we spend our
program time in.

### Criterion

Once we know _where_ to focus our optimization efforts, we need to measure the
impacts of our changes.

To do this, I chose to use [criterion-rs](https://lib.rs/crates/criterion).

You can look at the benchmarks in the `benches/` directory to see what kind of
workloads will be measured by the benchmarks.

To run the criterion benchmarks after making a modification, simply run

```bash
# Simply run the benchmarks
cargo bench

# Run the benchmark and save it as <new_baseline>
cargo bench --bench benchmark -- --save-baseline <new_baseline>

# Run the benchmark and compare the results against the <baseline>'s
cargo bench --bench benchmark -- --baseline <baseline>

# Don't run the benchmark but load the <new_baseline> and compare it to <old_baseline>
cargo bench --bench benchmark -- --load-baseline <new_baseline> --baseline <old_baseline>

# Run a specific benchmark
cargo bench --bench benchmark
```

Beware, this takes more than **5 minutes** to run the full benchmark.


# Improvements

- [ ] So far, these structures only store data in memory. This approach is not
      scalable, as the number of transactions grow. To make these structures more scalable, we could move the backing storage of the _TransactionLog_ to a database that is well tuned for writes, as we would most likely append data often, and query only for disputes.

- [ ] The Fixtures used for testing could be better. We could imagine a system 
      where we can create a simple list of transactions using macros. (transactions!(deposit!(1, 10.1), deposit!(2, 4.0), withdraw!(1, 44.134533))) -> Input Transactions to the engine
