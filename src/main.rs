use std::env;

use csv::{ ReaderBuilder, Trim };
use engine::Engine;
use tokio::{ io::{ stdout, AsyncWriteExt }, join, spawn, sync::mpsc };
use types::Transaction;

mod engine;
mod types;

const BUFFER_SIZE: usize = 100;

#[tokio::main]
async fn main() {
    let file = env::args().nth(1).expect("Specify the csv file");

    env_logger::init();

    log::info!("Starting...");

    let (tx, mut rx) = mpsc::channel::<Transaction>(BUFFER_SIZE);

    let file_input = spawn(async move {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(file)
            .expect("Could not open the csv file");

        for record in reader.deserialize::<Transaction>() {
            let Ok(transaction) = record else {
                log::error!("Failed to parse transaction");
                continue;
            };

            if tx.send(transaction).await.is_err() {
                log::error!("Failed to send transaction to engine");
                break;
            }
        }
    });

    let consume = spawn(async move {
        let mut engine = Engine::new();

        while let Some(transaction) = rx.recv().await {
            engine.add_transaction(transaction);
        }

        let mut writer = csv::Writer::from_writer(vec![]);

        engine
            .get_accounts()
            .into_iter()
            .for_each(|account| {
                let _ = writer.serialize(account);
            });

        if let Ok(bytes) = writer.into_inner() {
            let _ = stdout().write_all(&bytes).await;
        } else {
            log::error!("Failed to serialize accounts");
        }
    });

    let _ = join!(file_input, consume);
}
