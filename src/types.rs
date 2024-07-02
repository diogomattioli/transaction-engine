use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{ de, Deserialize, Serialize };

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Deserialize)]
// #[serde(untagged)]
// #[serde(tag = "type", content = "amount", rename_all = "lowercase")]
pub enum TransactionType {
    Deposit(Decimal),
    Withdrawal(Decimal),
    Dispute,
    Resolve,
    Chargeback,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    // #[serde(flatten)] //, deserialize_with = "custom_serde::deserialize_transaction_type")]
    #[serde(flatten, deserialize_with = "custom_serde::deserialize_transaction_type")]
    pub tx_type: TransactionType,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Debug, Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(serialize_with = "custom_serde::serialize_decimal")]
    pub available: Decimal,
    #[serde(serialize_with = "custom_serde::serialize_decimal")]
    pub held: Decimal,
    #[serde(serialize_with = "custom_serde::serialize_decimal")]
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Account {
            client_id,
            available: dec!(0),
            held: dec!(0),
            total: dec!(0),
            locked: false,
        }
    }
}

mod custom_serde {
    use std::fmt;

    use rust_decimal::prelude::FromPrimitive;
    use serde::{ Deserialize, Deserializer, Serializer };

    use super::*;

    const PRECISION: u32 = 4;

    pub fn serialize_decimal<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&value.round_dp(PRECISION).to_string())
    }

    fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
        where D: Deserializer<'de>
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a float as a string")
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: serde::de::Error {
                Decimal::from_f64(v).ok_or(de::Error::missing_field("amount"))
            }

            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
                Ok(Decimal::ZERO)
            }

            fn visit_string<E>(self, _v: String) -> Result<Self::Value, E> where E: serde::de::Error {
                Ok(Decimal::ZERO)
            }
        }

        deserializer.deserialize_any(Visitor)
    }

    pub fn deserialize_transaction_type<'a, D>(deserializer: D) -> Result<TransactionType, D::Error>
        where D: Deserializer<'a>
    {
        #[derive(Deserialize, Debug)]
        struct Helper {
            #[serde(rename = "type")]
            tx_type: String,
            #[serde(deserialize_with = "deserialize_decimal")]
            amount: Decimal,
        }

        let helper = Helper::deserialize(deserializer)?;

        match helper.tx_type.as_str() {
            "deposit" => Ok(TransactionType::Deposit(helper.amount.round_dp(PRECISION))),
            "withdrawal" => Ok(TransactionType::Withdrawal(helper.amount.round_dp(PRECISION))),
            "dispute" => Ok(TransactionType::Dispute),
            "resolve" => Ok(TransactionType::Resolve),
            "chargeback" => Ok(TransactionType::Chargeback),
            _ =>
                Err(
                    de::Error::unknown_variant(
                        helper.tx_type.as_str(),
                        &["deposit", "withdrawal", "dispute", "resolve", "chargeback"]
                    )
                ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn deserialize_deposit() {
        let input = "type,client,tx,amount\ndeposit,10,20,30.123\n";
        let mut reader = csv::Reader::from_reader(input.as_bytes());

        let tx = reader.deserialize::<Transaction>().next().unwrap().unwrap();
        assert_eq!(tx, Transaction {
            client_id: 10,
            tx_id: 20,
            tx_type: TransactionType::Deposit(dec!(30.123)),
        });
    }

    #[test]
    fn deserialize_withdrawal() {
        let input = "type,client,tx,amount\nwithdrawal,10,20,30.123\n";
        let mut reader = csv::Reader::from_reader(input.as_bytes());

        let tx = reader.deserialize::<Transaction>().next().unwrap().unwrap();
        assert_eq!(tx, Transaction {
            client_id: 10,
            tx_id: 20,
            tx_type: TransactionType::Withdrawal(dec!(30.123)),
        });
    }

    #[test]
    fn deserialize_dispute() {
        let input = "type,client,tx,amount\ndispute,10,20,\n";
        let mut reader = csv::Reader::from_reader(input.as_bytes());

        let tx = reader.deserialize::<Transaction>().next().unwrap().unwrap();
        assert_eq!(tx, Transaction {
            client_id: 10,
            tx_id: 20,
            tx_type: TransactionType::Dispute,
        });
    }

    #[test]
    fn deserialize_resolve() {
        let input = "type,client,tx,amount\nresolve,10,20,\n";
        let mut reader = csv::Reader::from_reader(input.as_bytes());

        let tx = reader.deserialize::<Transaction>().next().unwrap().unwrap();
        assert_eq!(tx, Transaction {
            client_id: 10,
            tx_id: 20,
            tx_type: TransactionType::Resolve,
        });
    }

    #[test]
    fn deserialize_chargeback() {
        let input = "type,client,tx,amount\nchargeback,10,20,\n";
        let mut reader = csv::Reader::from_reader(input.as_bytes());

        let tx = reader.deserialize::<Transaction>().next().unwrap().unwrap();
        assert_eq!(tx, Transaction {
            client_id: 10,
            tx_id: 20,
            tx_type: TransactionType::Chargeback,
        });
    }

    #[test]
    fn serialize_account() {
        let account = Account::new(1);

        let mut writer = csv::Writer::from_writer(vec![]);
        writer.serialize(account).unwrap();

        let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(output, "client,available,held,total,locked\n1,0,0,0,false\n");
    }

    #[test]
    fn serialize_account_with_scale() {
        let mut account = Account::new(1);
        account.available = dec!(0.123456789);
        account.locked = true;

        let mut writer = csv::Writer::from_writer(vec![]);
        writer.serialize(account).unwrap();

        let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(output, "client,available,held,total,locked\n1,0.1235,0,0,true\n");
    }
}
