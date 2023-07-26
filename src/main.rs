use std::{
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use cli::{Action, Args, ListenFlags, ProduceFlags};
use config::Config;
use fs::{File, FileError};
use rdkafka::{
    consumer::StreamConsumer,
    producer::{FutureProducer, FutureRecord},
};
use tokio::sync::Semaphore;

mod cli;
mod config;
mod fs;
mod log;

#[tokio::main]
async fn main() -> Result<(), FileError> {
    let subscriber = log::get_plain_subscriber();
    log::init_subscriber(subscriber);

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
        Action::Produce(ProduceFlags {
            file,
            key,
            topic,
            iterations,
        }) => {
            let producer: Arc<FutureProducer> = {
                let mut cfg = config.to_client_config();

                for (k, v) in args_map {
                    cfg.set(&k, &v);
                }

                Arc::new(cfg.create().expect("Failed to initialize producer"))
            };

            let payload = Arc::new(tokio::fs::read_to_string(Path::new(&file)).await?);
            let key = Arc::new(key.unwrap_or("test".to_string()));

            let semaphore = Arc::new(Semaphore::new(10));
            let timings = Arc::new(Mutex::new(Vec::with_capacity(iterations)));
            let avg_time = Arc::new(Mutex::new(Duration::from_secs(0)));

            let job_start = tokio::time::Instant::now();

            let mut handles = Vec::new();
            for i in 0..iterations {
                let handle = tokio::spawn({
                    let producer = Arc::clone(&producer);
                    let payload = Arc::clone(&payload);
                    let key = Arc::clone(&key);
                    let topic = Arc::new(topic.clone());
                    let semaphore = Arc::clone(&semaphore);
                    let start = tokio::time::Instant::now();
                    let timings = Arc::clone(&timings);
                    let avg_time = Arc::clone(&avg_time);

                    async move {
                        let permit = semaphore.acquire_owned().await.expect("Semaphore failed");

                        let message_result = producer
                            .send(
                                FutureRecord::to(&topic)
                                    .payload(&payload.to_string())
                                    .key(&key.to_string()),
                                Duration::from_secs(0),
                            )
                            .await;

                        let total_msgs = iterations;
                        let curr = i + 1;

                        let elapsed = start.elapsed();
                        let mut timings = timings.lock().unwrap();
                        let mut avg_time = avg_time.lock().unwrap();

                        timings.push(elapsed);
                        let timings_sum = timings.iter().sum::<Duration>();
                        *avg_time = timings_sum / timings.len() as u32;

                        let eta = *avg_time * (iterations - curr) as u32;

                        match message_result {
                            Ok(data) => {
                                print!(
                                    "{curr}/{total_msgs} [ETA: {eta:?}] - Message sent: {data:?}"
                                );
                                io::stdout().flush().unwrap();
                                print!("\r");
                            }
                            Err((e, _)) => {
                                tracing::error!("{curr}/{total_msgs} Failed to send message: {e}")
                            }
                        };

                        drop(permit);
                        drop(timings);
                        drop(avg_time);
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                if let Err(e) = handle.await {
                    tracing::error!("Failed to send message: {}", e);
                }
            }

            let job_elapsed = job_start.elapsed();
            tracing::info!("Sent {iterations}/{iterations} messages in {job_elapsed:?}");
        }
    }

    Ok(())
}
