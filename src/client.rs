//! Represents the Client data structure
use crate::error::{Error, Result};
use crate::transaction::Transaction;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;

/// Represents a Client's state
#[derive(Debug, Clone)]
pub struct Wallet {
    id: u16,
    available_balance: f64,
    held_balance: f64,
    total_balance: f64,
    locked: bool,
    transactions: HashMap<u32, Transaction>,
}

impl Serialize for Wallet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut client = serializer.serialize_struct("Client", 5)?;
        client.serialize_field("client", &self.id)?;
        client.serialize_field(
            "available",
            &((self.available_balance * 10000.0).round() / 10000.0),
        )?;
        client.serialize_field("held", &((self.held_balance * 10000.0).round() / 10000.0))?;
        client.serialize_field("total", &((self.total_balance * 10000.0).round() / 10000.0))?;
        client.serialize_field("locked", &self.locked)?;
        client.end()
    }
}

impl Wallet {
    /// Build an empty & unlocked wallet with an arbitrary `id`
    pub fn with_id(id: u16) -> Self {
        let c = Wallet {
            id,
            available_balance: 0.0,
            held_balance: 0.0,
            total_balance: 0.0,
            locked: false,
            transactions: HashMap::new(),
        };
        c
    }

    /// Check if the transaction is linked to this wallet
    pub fn has_tx(&self, tx_id: u32) -> bool {
        self.transactions.contains_key(&tx_id)
    }

    /// Get the details of the tx, if stored in this wallet
    pub fn get_tx(&self, tx_id: u32) -> Option<&Transaction> {
        self.transactions.get(&tx_id)
    }

    /// Get access to the tx, if stored in this wallet
    pub fn get_tx_mut(&mut self, tx_id: u32) -> Option<&mut Transaction> {
        self.transactions.get_mut(&tx_id)
    }

    /// record a transaction to the wallet's log
    pub fn record_tx(&mut self, t: &Transaction) {
        match self.transactions.insert(t.id, t.clone()) {
            Some(_) => panic!("Overwrotte a previous transaction in client log"),
            None => (),
        }
    }

    /// Add funds to the client's balance
    /// This function updates the client's available and total balance
    pub fn credit(&mut self, amount: f64) {
        self.available_balance += amount;
        self.total_balance += amount;
    }

    /// Take away from the client's balance
    /// This function updates the client's available and total balance
    ///
    /// # Note
    /// This function does prevent debiting more than the available balance
    pub fn debit(&mut self, amount: f64) -> Result<()> {
        if amount > self.available_balance {
            return Err(Error::InssuficientFunds);
        }
        self.available_balance -= amount;
        self.total_balance -= amount;
        Ok(())
    }

    /// Put some of the client's funds in holding
    /// This function updates the client's available, held and total balance
    pub fn hold(&mut self, amount: f64) -> Result<()> {
        if amount > self.available_balance {
            return Err(Error::InssuficientFunds);
        }
        self.available_balance -= amount;
        self.held_balance += amount;
        Ok(())
    }

    /// Put funds from holding back into the available balance
    pub fn release(&mut self, amount: f64) -> Result<()> {
        if self.held_balance < amount {
            return Err(Error::InssuficientFunds);
        }
        self.held_balance -= amount;
        self.available_balance += amount;
        Ok(())
    }

    /// Remove `amount` of funds from holding, deceasing total balance.
    pub fn confiscate(&mut self, amount: f64) -> Result<()> {
        if self.held_balance < amount {
            return Err(Error::InssuficientFunds);
        }
        self.held_balance -= amount;
        self.total_balance -= amount;
        Ok(())
    }

    /// Get the client's id
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Lock a client
    pub fn lock(&mut self) {
        self.locked = true;
    }

    // Getters for unit tests
    #[cfg(test)]
    pub fn available_balance(&self) -> f64 {
        self.available_balance
    }
    #[cfg(test)]
    pub fn held_balance(&self) -> f64 {
        self.held_balance
    }
    #[cfg(test)]
    pub fn total_balance(&self) -> f64 {
        self.total_balance
    }
}

#[derive(Debug, Clone)]
pub struct ClientWallets {
    wallets: HashMap<u16, Wallet>,
}

impl ClientWallets {
    pub fn new() -> Self {
        ClientWallets {
            wallets: HashMap::new(),
        }
    }

    pub fn get_or_create_mut(&mut self, client_id: u16) -> &mut Wallet {
        if !self.wallets.contains_key(&client_id) {
            self.wallets.insert(client_id, Wallet::with_id(client_id));
        }
        self.wallets.get_mut(&client_id).unwrap()
    }

    pub fn print_balances(&self) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());

        // We sort here to have a consistent output order.
        // This allows for easier testing, as more predictable.
        let mut sorted: Vec<(&u16, &Wallet)> = self.wallets.iter().collect();
        sorted.sort_by_key(|(_, c)| c.id());
        for (_, client) in &sorted {
            wtr.serialize(client)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn base_client() -> Wallet {
        Wallet::with_id(1)
    }

    /// construct a new client with a balance of `balance`
    fn base_client_with_funds(funds: f64) -> Wallet {
        let mut c = Wallet::with_id(1);
        c.available_balance += funds;
        c.total_balance += funds;
        c
    }

    #[test]
    /// Test that we credit a client the right amount.
    fn credit() {
        let mut client = base_client();
        client.credit(1.88889);
        assert_eq!(client.available_balance(), 1.88889);
        assert_eq!(client.total_balance(), 1.88889);
    }

    #[test]
    fn debit_no_funds() {
        let mut client = base_client();
        match client.debit(1.88889) {
            Err(Error::InssuficientFunds) => (),
            otherwise => panic!("{:?}", otherwise),
        }
        assert_eq!(client.available_balance(), 0.0);
        assert_eq!(client.total_balance(), 0.0);
    }

    #[test]
    fn debit_too_much() {
        let mut client = base_client_with_funds(19.0);
        match client.debit(50.9) {
            Err(Error::InssuficientFunds) => (),
            otherwise => panic!("{:?}", otherwise),
        }
        assert_eq!(client.available_balance(), 19.0);
        assert_eq!(client.total_balance(), 19.0);
    }

    #[test]
    fn debit() {
        let mut client = base_client_with_funds(19.0);
        client.debit(10.9).expect("Debit should have went through");
        assert_eq!(client.available_balance(), 8.1);
        assert_eq!(client.total_balance(), 8.1);
    }

    #[test]
    fn hold() {
        let mut client = base_client_with_funds(19.0);

        client
            .hold(10.0)
            .expect("Should have been able to hold funds");
        assert_eq!(client.held_balance(), 10.0);
        assert_eq!(client.available_balance(), 9.0);
        assert_eq!(client.total_balance(), 19.0);
    }

    #[test]
    fn hold_no_funds() {
        let mut client = base_client_with_funds(1.0);

        match client.hold(10.0) {
            Err(Error::InssuficientFunds) => (),
            otherwise => panic!("{:?}", otherwise),
        };
        assert_eq!(client.held_balance(), 0.0);
        assert_eq!(client.available_balance(), 1.0);
        assert_eq!(client.total_balance(), 1.0);
    }

    #[test]
    fn release() {
        let mut client = base_client_with_funds(19.0);

        client
            .hold(10.0)
            .expect("Should have been able to hold funds");
        assert_eq!(client.held_balance(), 10.0);
        assert_eq!(client.available_balance(), 9.0);
        assert_eq!(client.total_balance(), 19.0);
        client
            .release(10.0)
            .expect("Should have been able to release funds");
        assert_eq!(client.held_balance(), 0.0);
        assert_eq!(client.available_balance(), 19.0);
        assert_eq!(client.total_balance(), 19.0);
    }

    #[test]
    fn release_no_funds() {
        let mut client = base_client_with_funds(19.0);
        match client.release(10.0) {
            Err(Error::InssuficientFunds) => (),
            otherwise => panic!("{:?}", otherwise),
        }
        assert_eq!(client.held_balance(), 0.0);
        assert_eq!(client.available_balance(), 19.0);
        assert_eq!(client.total_balance(), 19.0);
    }
}
