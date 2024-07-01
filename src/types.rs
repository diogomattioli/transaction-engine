use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{ Deserialize, Serialize };

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Deserialize)]
pub struct Transaction {
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    tx_id: u32,
    #[serde(rename = "type")]
    tx_type: TransactionType,
    #[serde(deserialize_with = "decimal_serde::deserialize")]
    amount: Option<Decimal>,
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(serialize_with = "decimal_serde::serialize")]
    available: Decimal,
    #[serde(serialize_with = "decimal_serde::serialize")]
    held: Decimal,
    #[serde(serialize_with = "decimal_serde::serialize")]
    total: Decimal,
    locked: bool,
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

mod decimal_serde {
    use serde::{ Deserialize, Deserializer, Serializer };

    use super::*;

    const PRECISION: u32 = 4;

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut rounded_value = value.round_dp(PRECISION);

        if rounded_value.scale() == 0 {
            let _ = rounded_value.set_scale(1);
        }

        serializer.serialize_str(&rounded_value.to_string())
    }

    pub fn deserialize<'a, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
        where D: Deserializer<'a>
    {
        let str = String::deserialize(deserializer)?;
        if str.is_empty() {
            return Ok(None);
        }

        let value = str.parse::<Decimal>().map_err(serde::de::Error::custom)?;
        Ok(Some(value.round_dp(PRECISION)))
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
            amount: Some(dec!(30.123)),
            tx_type: TransactionType::Deposit,
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
            amount: Some(dec!(30.123)),
            tx_type: TransactionType::Withdrawal,
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
            amount: None,
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
            amount: None,
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
            amount: None,
            tx_type: TransactionType::Chargeback,
        });
    }

    #[test]
    fn serialize_account() {
        let account = Account::new(1);

        let mut writer = csv::Writer::from_writer(vec![]);
        writer.serialize(account).unwrap();

        let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(output, "client,available,held,total,locked\n1,0.0,0.0,0.0,false\n");
    }

    #[test]
    fn serialize_account_with_scale() {
        let mut account = Account::new(1);
        account.available = dec!(0.123456789);
        account.locked = true;

        let mut writer = csv::Writer::from_writer(vec![]);
        writer.serialize(account).unwrap();

        let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(output, "client,available,held,total,locked\n1,0.1235,0.0,0.0,true\n");
    }
}
