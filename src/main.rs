use std::{path::Path, time::Duration};

use clap::Parser;
use cli::{Action, Args, ListenFlags, ProduceFlags};
use config::Config;
use fs::{File, FileError};
use rdkafka::{
    consumer::StreamConsumer,
    producer::{FutureProducer, FutureRecord},
};

mod cli;
mod config;
mod fs;
mod log;

#[tokio::main]
async fn main() -> Result<(), FileError> {
    let subscriber = log::get_debug_subscriber("debug");
    log::init_subscriber(subscriber);

    tracing::info!("INFOOOOOOOOOOO");

    let args = Args::parse();

    let config: Config;

    if Config::exists().await {
        tracing::debug!("Config file exists ({}), loading...", config::CONFIG_FILE);
        config = Config::load(Path::new(config::CONFIG_FILE)).await?;
    } else {
        tracing::debug!(
            "Config file not found ({}), creating...",
            config::CONFIG_FILE
        );
        config = Config::new(args.brokers.clone());
        config.write(Path::new(config::CONFIG_FILE)).await?;
    }

    tracing::debug!("Config: {:?}", config);

    let args_map = args.to_map();

    match args.action {
        Action::Listen(ListenFlags {
            json: _,
            topic: _,
            group_id,
        }) => {
            let _consumer: StreamConsumer = {
                let mut cfg = config.to_client_config();

                if let Some(group_id) = group_id {
                    cfg.set("group.id", group_id);
                }

                for (k, v) in args_map {
                    cfg.set(&k, &v);
                }

                cfg.create().expect("Failed to initialize consumer")
            };

            todo!("Implement listen action");
        }
        Action::Produce(ProduceFlags { file, key, topic }) => {
            let producer: FutureProducer = {
                let mut cfg = config.to_client_config();

                for (k, v) in args_map {
                    cfg.set(&k, &v);
                }

                cfg.create().expect("Failed to initialize producer")
            };

            let payload = tokio::fs::read_to_string(Path::new(&file)).await?;
            let key = key.unwrap_or("test".to_string());

            tracing::info!("Sending message to topic: {}", topic);
            let message_result = producer
                .send(
                    FutureRecord::to(&topic).payload(&payload).key(&key),
                    Duration::from_secs(0),
                )
                .await;

            match message_result {
                Ok(data) => tracing::info!("Message sent successfully: {:?}", data),
                Err((e, _)) => tracing::error!("Failed to send message: {}", e),
            }
        }
    }

    Ok(())
}
