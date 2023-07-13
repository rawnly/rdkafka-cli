use std::collections::HashMap;

use crate::fs::File;
use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};

/// DEFAULT CONFIG FILE NAME
pub const CONFIG_FILE: &str = "rdkafka-config.yml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub connection: HashMap<String, String>,
    pub producer: HashMap<String, String>,
    pub consumer: HashMap<String, String>,
}

impl Config {
    pub fn new(brokers: Vec<String>) -> Self {
        let producer: HashMap<String, String> = HashMap::new();
        let consumer: HashMap<String, String> = HashMap::new();
        let mut connection: HashMap<String, String> = HashMap::new();

        connection.insert("bootstrap.servers".to_string(), brokers.join(","));

        Config {
            producer,
            consumer,
            connection,
        }
    }

    pub async fn exists() -> bool {
        tokio::fs::try_exists(CONFIG_FILE).await.unwrap_or(false)
    }

    pub fn to_client_config(self) -> ClientConfig {
        self.into()
    }
}

impl Into<ClientConfig> for Config {
    fn into(self) -> ClientConfig {
        let mut config = ClientConfig::new();

        for (key, value) in self.connection {
            config.set(key, value);
        }

        config
    }
}

impl File for Config {}
