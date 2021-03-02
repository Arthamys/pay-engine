use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type of the transaction (withdrawal, deposit, dispute, resolve, chargeback)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Withdrawal,
    Deposit,
    Dispute,
    Resolve,
    Chargeback,
}

/// A Transaction record
#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    pub r#type: Type,
    pub client: u16,
    pub id: u32,
    #[serde(default)]
    pub amount: f64,
    #[serde(skip_deserializing)]
    under_dispute: bool,
}

impl Transaction {
    /// Check if the transaction is under dispute
    pub fn under_dispute(&self) -> bool {
        self.under_dispute
    }
}

/// The TransactionLog holds the list of all valid transactions processed
/// by the engine.
/// Every transaction that gets successfully processed by the engine gets
/// `push()`ed to the log.
/// It can then be queried to `find()` a specific transaction by id.
/// Attempting to add a transaction with an id that already was recorded
/// silently fails.
#[derive(Debug)]
pub struct TransactionLog {
    /// map of transaction id to transaction
    transactions: HashMap<u32, Transaction>,
}

impl TransactionLog {
    /// Create a new empty transaction log
    pub fn new() -> Self {
        TransactionLog {
            transactions: HashMap::new(),
        }
    }

    /// Add a new transaction to the list
    pub fn push(&mut self, t: &Transaction) {
        match self.transactions.get(&t.id) {
            Some(_) => (), // silently fail
            None => {
                self.transactions.insert(t.id, t.clone());
            }
        };
    }

    /// Find a transaction with a given id in the log
    pub fn find(&self, tx_id: u32) -> Option<&Transaction> {
        self.transactions.get(&tx_id)
    }

    /// Checks if the transaction with id `tx_id` exits in the log
    pub fn contains(&self, tx_id: u32) -> bool {
        self.transactions.contains_key(&tx_id)
    }

    /// Mark a transaction as under dispute
    pub fn dispute(&mut self, tx_id: u32) {
        match self.transactions.get_mut(&tx_id) {
            None => (),
            Some(t) => t.under_dispute = true,
        }
    }

    /// Reset the dispute status of a transaction
    pub fn undispute(&mut self, tx_id: u32) {
        match self.transactions.get_mut(&tx_id) {
            None => (),
            Some(t) => t.under_dispute = false,
        }
    }
}
