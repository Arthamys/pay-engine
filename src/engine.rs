use crate::client::Client;
use crate::transaction::{Transaction, TransactionLog, Type::*};

/// Run the correct logic for the type of transaction.
/// If the transaction was valid and successful, it gets added to the TransactionLog
pub fn execute_transaction(t: &Transaction, t_log: &mut TransactionLog, client: &mut Client) {
    let tx_exists = t_log.contains(t.id);
    let record_op = match t.r#type {
        Deposit if !tx_exists => deposit(client, t.amount),
        Withdrawal if !tx_exists => withdraw(client, t.amount),
        Dispute => dispute(client, t.id, t_log),
        Resolve => resolve(client, t.id, t_log),
        Chargeback => chargeback(client, t.id, t_log),
        _ => false,
    };
    if record_op {
        t_log.push(t);
    }
}

/// Credit the client's account of `amount` funds.
fn deposit(client: &mut Client, amount: f64) -> bool {
    client.credit(amount);
    log::trace!("deposited {} to client {}'s balance", amount, client.id());
    true
}

/// Withdraw `amount` from the client's account, if there are sufficient funds.
fn withdraw(client: &mut Client, amount: f64) -> bool {
    if let Err(_) = client.debit(amount) {
        log::debug!(
            "client {} tried to withdraw more than available balance",
            client.id()
        );
        return false;
    }
    log::trace!("withdrew {} from client {}'s balance", amount, client.id());
    true
}

/// Dispute a transaction
///
/// # Notes:
/// - Disputing an order can only be done by the client that has issued the
/// target transaction.
/// - A transaction can only be under dispute once at a time. If a dispute is
/// opened on a transaction, subsequent disputes will have no effect.
/// - If a dispute would engage funds that are no longer available, nothing happens
/// - If there is no record of transaction `tx`, nothing happens
fn dispute(client: &mut Client, tx: u32, tx_hist: &mut TransactionLog) -> bool {
    // check that the target transaction exists
    match tx_hist.find(tx) {
        Some(transaction) => {
            if transaction.client != client.id() {
                log::debug!(
                    "dispute started by unauthorized client (offending client: {}",
                    transaction.client
                );
                return false;
            }
            // make sure the transaction was issued by the client making the
            // dispute request
            if !transaction.under_dispute() {
                // hold the client's funds
                if let Err(_) = client.hold(transaction.amount) {
                    log::debug!(
                        "Inssuficient funds to dispute transaction {}",
                        transaction.id
                    );
                    return false;
                }
                // mark transaction as under dispute
                tx_hist.dispute(tx);
            } else {
                log::debug!("transaction {} is already under dispute", tx);
                return false;
            }
        }
        None => {
            log::debug!("Invalid transaction number");
            return false;
        } // Invalid transaction number
    }
    log::trace!(
        "opening dispute on transaction {} made by client {}",
        tx,
        client.id()
    );
    true
}

/// Resolve a transaction
/// A transaction can only be resolved by whoever issued it.
/// If the transaction is not under dispute, it does nothing.
fn resolve(client: &mut Client, tx: u32, tx_hist: &mut TransactionLog) -> bool {
    match tx_hist.find(tx) {
        Some(transaction) => {
            // make sure the transaction was issued by the client making this request
            if client.id() != transaction.client {
                log::warn!("unauthorized client tried to resolve transaction {}", tx);
                return false;
            }
            if transaction.under_dispute() {
                if let Err(_) = client.release(transaction.amount) {
                    log::warn!("Insufficient held funds to resolve transaction {}", tx);
                    return false;
                }
                tx_hist.undispute(tx);
            } else {
                log::debug!(
                    "transaction {} isn't under dispute. It cannot be resolved.",
                    tx
                );
                return false;
            }
        }
        None => {
            log::debug!("Invalid transaction number");
            return false;
        }
    }
    log::trace!(
        "resolving dispute on transaction {} made by client {}",
        tx,
        client.id()
    );
    true
}

fn chargeback(client: &mut Client, tx: u32, tx_hist: &mut TransactionLog) -> bool {
    match tx_hist.find(tx) {
        Some(transaction) => {
            // make sure the transaction was issued by the client making the
            // dispute request
            if transaction.under_dispute() {
                if let Err(_) = client.confiscate(transaction.amount) {
                    log::warn!("Inssuficient funds to chargeback transaction {}", tx);
                    return false;
                }
                // Uncomment to allow a transaction to be disputed multiple times.
                //tx_hist.resolve(tx);
            } else {
                log::debug!("Transaction {} is not under dispute", tx);
                return false;
                // invalid dispute order
            }
        }
        None => {
            log::debug!("Invalid transaction number");
            return false;
        }
    }

    client.lock();
    log::trace!(
        "charging back dispute on transaction {} made by client {}",
        tx,
        client.id()
    );
    true
}
