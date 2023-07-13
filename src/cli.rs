use std::{collections::HashMap, str::FromStr};

use clap::{Parser, Subcommand};

#[derive(Debug, Clone)]
pub enum SecurityProtocol {
    Plaintext,
    Ssl,
    SaslPlaintext,
    SaslSsl,
}

impl Default for SecurityProtocol {
    fn default() -> Self {
        Self::Plaintext
    }
}

impl FromStr for SecurityProtocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PLAINTEXT" => Ok(Self::Plaintext),
            "SSL" => Ok(Self::Ssl),
            "SASL_PLAINTEXT" => Ok(Self::SaslPlaintext),
            "SASL_SSL" => Ok(Self::SaslSsl),
            _ => Err(format!("Invalid security protocol: {}", s)),
        }
    }
}

impl ToString for SecurityProtocol {
    fn to_string(&self) -> String {
        match self {
            SecurityProtocol::Plaintext => "PLAINTEXT".to_string(),
            SecurityProtocol::Ssl => "SSL".to_string(),
            SecurityProtocol::SaslPlaintext => "SASL_PLAINTEXT".to_string(),
            SecurityProtocol::SaslSsl => "SASL_SSL".to_string(),
        }
    }
}
#[derive(Parser, Debug, Clone)]
#[clap(author, version)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,

    #[clap(long, global = true, default_value = "6")]
    /// The log level for rdkafka
    pub kafka_log_level: u8,

    #[clap(long, global = true, default_value = "localhost:9092")]
    pub brokers: Vec<String>,

    #[clap(long, global = true)]
    pub security_protocol: Option<SecurityProtocol>,

    #[clap(long, global = true)]
    pub sasl_username: Option<String>,

    #[clap(long, global = true)]
    pub sasl_password: Option<String>,

    #[clap(long, global = true)]
    pub sasl_mechanism: Option<String>,
}

impl Args {
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::<String, String>::new();

        if let Some(username) = &self.sasl_username {
            map.insert("sasl.username".to_string(), username.to_owned());
        }

        if let Some(password) = &self.sasl_password {
            map.insert("sasl.password".to_string(), password.to_owned());
        }

        if let Some(mechanism) = &self.sasl_mechanism {
            map.insert("sasl.mechanism".to_string(), mechanism.to_owned());
        }

        if let Some(security_protocol) = &self.security_protocol {
            map.insert(
                "security.protocol".to_string(),
                security_protocol.to_string(),
            );
        }

        map
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum Action {
    Listen(ListenFlags),
    Produce(ProduceFlags),
}

#[derive(Parser, Debug, Clone)]
pub struct ListenFlags {
    #[clap(long)]
    pub group_id: Option<String>,

    #[clap(long)]
    pub topic: String,

    #[clap(long, default_value = "false")]
    /// Try to parse json to pretty print
    pub json: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct ProduceFlags {
    #[clap(long)]
    pub file: String,

    #[clap(long)]
    pub topic: String,

    /// The key to use for the message
    pub key: Option<String>,
}
