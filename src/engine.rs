use std::time::Instant;

use crate::transaction::{Transaction, TransactionLog, Type::*};
use crate::Result;
use crate::{
    client::{ClientWallets, Wallet},
    error::Error,
};

pub struct Engine {
    past_tx: TransactionLog,
    wallets: ClientWallets,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            past_tx: TransactionLog::new(),
            wallets: ClientWallets::new(),
        }
    }

    pub fn run<S: Iterator>(
        &mut self,
        transactions: &mut S, /*stream of transactions*/
    ) -> ClientWallets
    where
        S::Item: Into<Transaction>,
    {
        let total = Instant::now();
        let total_tx_count = transactions.size_hint().1.unwrap_or(1) as u128;

        for t in transactions {
            let t = t.into();
            if t.primitive() && self.past_tx.contains(t.id) {
                // This is a duplicate transaction, ignore
                continue;
            }
            let single = Instant::now();
            let _ = execute_transaction(&t, self.wallets.get_or_create_mut(t.client))
                // handle errors in execution
                .map_err(|e| log::warn!("{}", e))
                .map(|r| {
                    if r.is_some() {
                        self.past_tx.push(r.unwrap());
                    }
                });
            log::trace!(
                "Took {}ns to process transaction",
                single.elapsed().as_nanos()
            );
        }
        // TODO delete
        log::error!(
            "Took ~{}ns per transaction",
            total.elapsed().as_nanos() / total_tx_count
        );
        self.wallets.clone()
    }

    /// Returns the number of transactions that were successfully executed and recorded
    /// by the engine.
    pub fn valid_transactions(&self) -> usize {
        self.past_tx.len()
    }
}

/// Given a transaction and the client that issued the transaction,
/// execute the transaction, if the Transaction was valid.
/// Returns the transaction if it needs to be recorded.
fn execute_transaction<'a>(
    t: &'a Transaction,
    wallet: &mut Wallet,
) -> Result<Option<&'a Transaction>> {
    match t.r#type {
        Deposit => deposit(t, wallet)?,
        Withdrawal => withdraw(t, wallet)?,
        Dispute => dispute(t, wallet)?,
        Resolve => resolve(t, wallet)?,
        Chargeback => chargeback(t, wallet)?,
    };
    Ok(Some(t))
}

fn deposit(tx: &Transaction, wallet: &mut Wallet) -> Result<()> {
    wallet.credit(tx.amount);
    wallet.record_tx(tx);
    Ok(())
}

fn withdraw(tx: &Transaction, wallet: &mut Wallet) -> Result<()> {
    wallet.debit(tx.amount)?;
    wallet.record_tx(tx);
    Ok(())
}

fn dispute(tx: &Transaction, wallet: &mut Wallet) -> Result<()> {
    let orig_tx = wallet
        .get_tx(tx.id)
        .ok_or(Error::AccessViolation(tx.client, tx.id))?
        .clone();
    if orig_tx.under_dispute() {
        Err(Error::MultipleDispute(tx.id))
    } else {
        wallet.get_tx_mut(tx.id).unwrap().dispute();
        wallet.hold(orig_tx.amount)
    }
}

fn resolve(tx: &Transaction, wallet: &mut Wallet) -> Result<()> {
    let orig_tx = wallet
        .get_tx(tx.id)
        .ok_or(Error::AccessViolation(tx.client, tx.id))?
        .clone();
    if orig_tx.under_dispute() {
        wallet.get_tx_mut(tx.id).unwrap().undispute();
        wallet.release(orig_tx.amount)
    } else {
        Err(Error::ResolveUndisputed(tx.client, tx.id))
    }
}

fn chargeback(tx: &Transaction, wallet: &mut Wallet) -> Result<()> {
    let orig_tx = wallet
        .get_tx(tx.id)
        .ok_or(Error::AccessViolation(tx.client, tx.id))?
        .clone();
    if orig_tx.under_dispute() {
        wallet.confiscate(orig_tx.amount)?;
        wallet.lock();
        Ok(())
    } else {
        Err(Error::ChargebackUndisputed(tx.client, tx.id))
    }
}
