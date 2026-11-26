//! Apache Kafka driver for `Broker`.
//!
//! Available when the `kafka` feature is enabled.
//! Backed by `rskafka` — a 100% pure Rust async Kafka client with zero C/C++ dependencies.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use rskafka::client::partition::{Compression, PartitionClient, UnknownTopicHandling};
use rskafka::client::{Client, ClientBuilder};
use rskafka::record::Record;

use crate::envelope::BrokerMessage;
use crate::error::{BrokerError, Result};
use crate::traits::{Broker, Subscription};

/// Kafka subscription wrapper polling a partition.
pub struct KafkaSubscription {
    topic: String,
    partition_client: Arc<PartitionClient>,
    next_offset: i64,
    buffered_records: Vec<BrokerMessage>,
}

#[async_trait]
impl Subscription for KafkaSubscription {
    async fn next_message(&mut self) -> Option<BrokerMessage> {
        loop {
            if !self.buffered_records.is_empty() {
                return Some(self.buffered_records.remove(0));
            }

            match self.partition_client.fetch_records(self.next_offset, 1..1024 * 1024, 1000).await
            {
                Ok((records, _watermark)) => {
                    if records.is_empty() {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        continue;
                    }

                    for record_and_offset in records {
                        self.next_offset = record_and_offset.offset + 1;
                        if let Some(val) = record_and_offset.record.value {
                            let msg = serde_json::from_slice::<BrokerMessage>(&val)
                                .unwrap_or_else(|_| BrokerMessage::new(&self.topic, val));
                            self.buffered_records.push(msg);
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Kafka fetch error on topic {}: {e}", self.topic);
                    tokio::time::sleep(Duration::from_millis(200)).await;
                },
            }
        }
    }
}

/// A message broker backed by Apache Kafka.
#[derive(Clone)]
pub struct KafkaBroker {
    client: Arc<Client>,
    correlation_seq: Arc<AtomicU64>,
}

impl KafkaBroker {
    /// Connects to Apache Kafka bootstrap brokers (e.g. `vec!["127.0.0.1:9092".to_string()]`).
    pub async fn new(bootstrap_brokers: Vec<String>) -> Result<Self> {
        let client = ClientBuilder::new(bootstrap_brokers)
            .build()
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
            correlation_seq: Arc::new(AtomicU64::new(1)),
        })
    }
}

#[async_trait]
impl Broker for KafkaBroker {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let partition_client = self
            .client
            .partition_client(topic, 0, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;

        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let timestamp =
            rskafka::chrono::DateTime::from_timestamp_millis(now_millis).unwrap_or_default();

        let record = Record {
            key: None,
            value: Some(payload.to_vec()),
            headers: BTreeMap::new(),
            timestamp,
        };

        partition_client
            .produce(vec![record], Compression::NoCompression)
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;

        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<Box<dyn Subscription>> {
        let partition_client = self
            .client
            .partition_client(topic, 0, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| BrokerError::Transport(e.to_string()))?;

        let offset = partition_client
            .get_offset(rskafka::client::partition::OffsetAt::Latest)
            .await
            .unwrap_or(0);

        Ok(Box::new(KafkaSubscription {
            topic: topic.to_string(),
            partition_client: Arc::new(partition_client),
            next_offset: offset,
            buffered_records: Vec::new(),
        }))
    }

    async fn request(&self, topic: &str, payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
        let seq = self.correlation_seq.fetch_add(1, Ordering::Relaxed);
        let reply_topic = format!("_kafka.reply.{seq}");
        let correlation_id = format!("corr-{seq}");

        let mut sub = self.subscribe(&reply_topic).await?;

        let request_msg = BrokerMessage::new(topic, payload.to_vec())
            .with_reply_to(&reply_topic)
            .with_correlation_id(&correlation_id);

        self.publish_message(&request_msg).await?;

        let result = tokio::time::timeout(timeout, async {
            while let Some(msg) = sub.next_message().await {
                if msg.correlation_id() == Some(&correlation_id) {
                    return Ok(msg.payload);
                }
            }
            Err(BrokerError::SubscriptionClosed)
        })
        .await;

        match result {
            Ok(inner) => inner,
            Err(_) => Err(BrokerError::Timeout(format!(
                "timeout waiting for Kafka reply on {topic} (corr: {correlation_id})"
            ))),
        }
    }
}
