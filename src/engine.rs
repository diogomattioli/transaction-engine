use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::types::{ Account, Transaction, TransactionType };

enum TransactionInfo {
    Regular,
    UnderDispute,
}

pub struct Engine {
    accounts: HashMap<u16, Account>,
    history: HashMap<u32, (TransactionInfo, Decimal)>,
}

impl Engine {
    pub fn new() -> Self {
        Engine { accounts: HashMap::new(), history: HashMap::new() }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        let account = self.accounts.entry(tx.client_id).or_insert(Account::new(tx.client_id));

        log::info!("{:?}", tx);

        match tx.tx_type {
            TransactionType::Deposit(amount) => {
                account.available += amount;
                self.history.insert(tx.tx_id, (TransactionInfo::Regular, amount));

                log::debug!("Successfull deposit of {}", amount);
            }
            TransactionType::Withdrawal(amount) => {
                if account.available >= amount {
                    account.available -= amount;

                    log::debug!("Successfull withdraw of {}", amount);
                }
            }
            TransactionType::Dispute => {
                if let Some((TransactionInfo::Regular, amount)) = self.history.get(&tx.tx_id) {
                    if account.available >= *amount {
                        account.available -= *amount;
                        account.held += *amount;

                        log::debug!("Successfull dispute of {} {}", tx.tx_id, *amount);

                        self.history.insert(tx.tx_id, (TransactionInfo::UnderDispute, *amount));
                    }
                }
            }
            TransactionType::Resolve => {
                if let Some((TransactionInfo::UnderDispute, amount)) = self.history.get(&tx.tx_id) {
                    account.available += *amount;
                    account.held -= *amount;

                    log::debug!("Successfull resolve of {} {}", tx.tx_id, *amount);

                    self.history.remove(&tx.tx_id);
                }
            }
            TransactionType::Chargeback => {
                if let Some((TransactionInfo::UnderDispute, amount)) = self.history.get(&tx.tx_id) {
                    account.held -= *amount;
                    account.locked = true;

                    log::debug!("Successfull chargeback of {} {}", tx.tx_id, *amount);

                    self.history.remove(&tx.tx_id);
                }
            }
        }

        account.total = account.available + account.held;
    }

    pub fn get_accounts(self) -> Vec<Account> {
        self.accounts.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::types::TransactionType;

    #[test]
    fn test_example() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(1.0)),
        });

        engine.add_transaction(Transaction {
            client_id: 2,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(2.0)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 3,
            tx_type: TransactionType::Deposit(dec!(2.0)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 4,
            tx_type: TransactionType::Withdrawal(dec!(1.5)),
        });

        engine.add_transaction(Transaction {
            client_id: 2,
            tx_id: 5,
            tx_type: TransactionType::Withdrawal(dec!(3.0)),
        });

        let accounts = engine.get_accounts();

        assert_eq!(accounts.len(), 2);

        assert_eq!(accounts[0], Account {
            client_id: 1,
            available: dec!(1.5),
            held: dec!(0),
            total: dec!(1.5),
            locked: false,
        });

        assert_eq!(accounts[1], Account {
            client_id: 2,
            available: dec!(2.0),
            held: dec!(0),
            total: dec!(2.0),
            locked: false,
        });
    }

    #[test]
    fn test_deposit() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(10));
    }

    #[test]
    fn test_withdrawal() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Withdrawal(dec!(5)),
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(5));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(5));
    }

    #[test]
    fn test_withdrawal_not_enough_funds() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Withdrawal(dec!(15)),
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(10));
    }

    #[test]
    fn test_dispute() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(5));
        assert_eq!(account.total, dec!(15));
    }

    #[test]
    fn test_dispute_not_enough_funds() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Withdrawal(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Dispute,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(5));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(5));
    }

    #[test]
    fn test_dispute_unknown() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 3,
            tx_type: TransactionType::Dispute,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(15));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(15));
    }

    #[test]
    fn test_resolve() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Resolve,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(15));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(15));
    }

    #[test]
    fn test_resolve_not_under_dispute() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Resolve,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(5));
        assert_eq!(account.total, dec!(15));
    }

    #[test]
    fn test_resolve_unknown() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 3,
            tx_type: TransactionType::Resolve,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(5));
        assert_eq!(account.total, dec!(15));
    }

    #[test]
    fn test_chargeback() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Chargeback,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(10));
    }

    #[test]
    fn test_chargeback_not_under_dispute() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Chargeback,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(5));
        assert_eq!(account.total, dec!(15));
    }

    #[test]
    fn test_chargeback_unknown() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 3,
            tx_type: TransactionType::Chargeback,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(5));
        assert_eq!(account.total, dec!(15));
    }

    #[test]
    fn test_get_accounts() {
        let mut engine = Engine::new();

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 1,
            tx_type: TransactionType::Deposit(dec!(10)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Deposit(dec!(5)),
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Dispute,
        });

        engine.add_transaction(Transaction {
            client_id: 1,
            tx_id: 2,
            tx_type: TransactionType::Chargeback,
        });

        let account = engine.get_accounts().pop().unwrap();
        assert_eq!(account.available, dec!(10));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(10));
    }
}
