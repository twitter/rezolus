use std::fs;
use std::io::Write;
use std::time::Duration;

use anyhow::{anyhow, Context};
use rdkafka::ClientConfig;
use serde::Deserialize;
use zookeeper::{WatchedEvent, Watcher, ZooKeeper};

fn default_endpoint() -> String {
    "kafka".to_owned()
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct KafkaConfig {
    pub zk_path: String,
    pub zk_server: String,
    #[serde(default = "default_endpoint")]
    pub zk_endpoint_name: String,

    pub topic: String,
}

struct ExitWatcher;
impl Watcher for ExitWatcher {
    fn handle(&self, _event: WatchedEvent) {
        std::process::exit(2);
    }
}

impl KafkaConfig {
    pub fn endpoints(&self) -> anyhow::Result<Vec<String>> {
        use serde_json::Value as JsonValue;

        debug!("Resolving kafka brokers from ZooKeeper");

        let zk = ZooKeeper::connect(&self.zk_server, Duration::from_secs(15), ExitWatcher)
            .with_context(|| format!("Unable to connect to zookeeper at {}", self.zk_server))?;

        let children = zk.get_children(&self.zk_path, true).with_context(|| {
            format!(
                "Unable to read children of zookeeper node '{}'",
                self.zk_path
            )
        })?;

        let mut endpoints = Vec::new();
        for child in children {
            let path = format!("{}/{}", self.zk_path, child);
            let data = zk
                .get_data(&path, false)
                .with_context(|| format!("Unable to get data of child node {}", path))?;
            let entry: JsonValue =
                serde_json::from_slice(&data.0).context("zknode data contained invalid json")?;
            let endpoint = &entry["additionalEndpoints"]["kafka-tls"];

            let host = match &endpoint["host"] {
                JsonValue::String(host) => &**host,
                _ => continue,
            };
            let port = match &endpoint["port"] {
                JsonValue::Number(port) => port,
                _ => continue,
            };
            debug!("Add broker {}:{}", host, port);
            endpoints.push(format!("{}:{}", host, port));
        }

        if endpoints.is_empty() {
            return Err(anyhow!(
                "Unable to find any kafka endpoints within ZooKeeper"
            ));
        }

        Ok(endpoints)
    }

    pub fn client_config(&self) -> anyhow::Result<ClientConfig> {
        let mut config = ClientConfig::new();
        let endpoints = self
            .endpoints()
            .context("Unable to resolve kafka broker endpoints")?;
        // We need to bundle the two certificates together to a file and pass it to
        // librdkafka
        let cert1_contents = fs::read_to_string("/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem")
                                .expect("Failed to read the public certificate file /etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem");
        let cert2_contents = fs::read_to_string("/usr/local/config/credential-service/prod/cfssl/int-bundle.crt")
                                .expect("Failed to read the public certificate file /usr/local/config/credential-service/prod/cfssl/int-bundle.crt");
        let mut cert_file = fs::File::create("/tmp/kafka-broker.pem").expect(
            "Failed to create the Kafka broker public certificate file /tmp/kafka-broker.pem",
        );
        cert_file
            .write_all(cert1_contents.as_bytes())
            .expect("Failed to write certificates to /tmp/kafka-broker.pem");
        cert_file
            .write_all(cert2_contents.as_bytes())
            .expect("Failed to write certificates to /tmp/kafka-broker.pem");
        config.set("security.protocol", "SASL_SSL");
        config.set("ssl.ca.location", "/tmp/kafka-broker.pem");
        config.set("sasl.mechanism", "GSSAPI");
        config.set("sasl.kerberos.service.name", "kafka");
        config.set("sasl.kerberos.principal", "fleetwide-profiling@TWITTER.BIZ");
        config.set(
            "sasl.kerberos.keytab",
            "/var/lib/tss/keys/fluffy/keytabs/client/fleetwide-profiling.keytab",
        );
        config.set("compression.codec", "gzip");
        config.set("metadata.broker.list", endpoints.join(","));
        config.set("statistics.interval.ms", "1000");
        config.set("queue.buffering.max.ms", "100");

        Ok(config)
    }
}
