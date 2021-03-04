use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The type of the transaction (withdrawal, deposit, dispute, resolve, chargeback)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Withdrawal,
    Deposit,
    Dispute,
    Resolve,
    Chargeback,
}

impl Type {
    fn random() -> Self {
        use Type::*;
        match random::<u8>() % 100_u8 {
            0..=29 => Withdrawal,
            30..=59 => Deposit,
            60..=69 => Dispute,
            70..=89 => Resolve,
            _ => Chargeback,
        }
    }
}

/// A Transaction record
#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Transaction {
    pub r#type: Type,
    pub client: u16,
    pub id: u32,
    #[serde(default)]
    pub amount: f64,
    #[serde(skip_deserializing)]
    under_dispute: bool,
}

static mut ID: u32 = 1;

impl Transaction {
    /// Create a new transaction filled with random data.
    pub fn new_random() -> Self {
        let t = Transaction {
            r#type: Type::random(),
            client: rand::random(),
            id: unsafe { ID.wrapping_sub(random()) },
            amount: rand::random(),
            under_dispute: rand::random(),
        };
        // Change the ID 30% of the time, to allow for generating
        // more plausible scenarios
        unsafe {
            if (random::<u8>() % 100) < 30 {
                ID += 1;
            }
        }
        t
    }

    /// Check if the transaction is under dispute
    pub fn under_dispute(&self) -> bool {
        self.under_dispute
    }

    /// Flag a transaction as under dispute
    pub fn dispute(&mut self) {
        self.under_dispute = true;
    }

    /// Clear the under dispute flag
    pub fn undispute(&mut self) {
        self.under_dispute = false;
    }

    /// Return true if the transaction is a primitive Deposit/withdrawal
    pub fn primitive(&self) -> bool {
        match self.r#type {
            Type::Deposit | Type::Withdrawal => true,
            _ => false,
        }
    }
}

/// The TransactionLog holds the list of all valid transactions processed
/// by the engine.
/// Every transaction that gets successfully processed by the engine gets
/// `push()`ed to the log.
/// Attempting to add a transaction with an id that already was recorded
/// silently fails.
#[derive(Debug)]
pub struct TransactionLog {
    /// map of transaction id to transaction
    transactions: HashSet<u32>,
}

impl TransactionLog {
    /// Create a new empty transaction log
    pub fn new() -> Self {
        TransactionLog {
            transactions: HashSet::new(),
        }
    }

    /// Returns the number of transactions in the log
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Add a new transaction to the list
    pub fn push(&mut self, t: &Transaction) {
        self.transactions.insert(t.id);
    }

    /// Checks if the transaction with id `tx_id` exits in the log
    pub fn contains(&self, tx_id: u32) -> bool {
        self.transactions.contains(&tx_id)
    }
}

pub mod utils {
    use super::Transaction;

    pub struct RandomTransactions {}

    impl RandomTransactions {
        pub fn new() -> Self {
            RandomTransactions {}
        }
    }

    impl Iterator for RandomTransactions {
        type Item = Transaction;

        fn next(&mut self) -> Option<Self::Item> {
            let tx = Transaction::new_random();
            Some(tx)
        }
    }
}
