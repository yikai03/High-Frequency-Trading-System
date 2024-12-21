use crate::stock::Stock;
use crate::order_dummy::OrderRequest;
use colored::Colorize;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, StreamConsumer, Consumer};
use rdkafka::producer::{ FutureProducer, FutureRecord};
use rdkafka::message::Message;
use rdkafka::util::Timeout;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};


pub struct KafkaConfig {
    pub producer: FutureProducer,
    pub consumer: Arc<Mutex<StreamConsumer>>, 
}

impl KafkaConfig{
    pub fn new(brokers: &str, group_id: &str) -> KafkaConfig {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
            .expect("Consumer creation error");

        KafkaConfig { producer, consumer:Arc::new(Mutex::new(consumer)) }
    }

    pub async fn producer_task(
        &self,
        topic: &str,
        mut stock_receiver: mpsc::Receiver<Stock>,
    ) {
        while let Some(stock) = stock_receiver.recv().await {
            let stock = Stock {
                name: stock.name.clone(),
                price: stock.price,
            };
            let payload = serde_json::to_string(&stock).unwrap();
            let record = FutureRecord::to(topic).payload(&payload).key("stock-key");

            match self.producer.send(record, Timeout::Never).await {
                Ok(_) => (),
                Err(e) => eprintln!("[{}]: Failed to send stock: {:?}", "kafka -- producer_task".red(),e),
            }
        }
    }

    pub async fn producer_task_for_order(
        &self,
        topic: &str,
        mut order_receiver: mpsc::Receiver<OrderRequest>,
    ) {
        while let Some(order) = order_receiver.recv().await {
            let payload = serde_json::to_string(&order).unwrap();
            let record = FutureRecord::to(topic).payload(&payload).key("order-key");

            match self.producer.send(record, Timeout::Never).await {
                Ok(_) => (),
                Err(e) => eprintln!("[{}]: Failed to send order: {:?}", "kafka -- producer_task_for_order".red(),e),
            }
        }
    }
    
    pub async fn consumer_order_task(
        &self,
        risk_sender: mpsc::Sender<String>,
    ) {
        let consumer = self.consumer.clone(); // Clone the Arc to access the consumer
        let consumer = consumer.lock().await; // Lock the consumer to safely access it
    
        // Subscribe to the topic asynchronously
        consumer.subscribe(&["orders"]).expect("Failed to subscribe to topic");
    
        let mut interval = interval(Duration::from_millis(50)); // For polling at regular intervals
        println!("Consumer started");
    
        // Start polling messages from Kafka
        loop {
            interval.tick().await; // Wait for the next polling interval
    
            // Poll for a message
            match consumer.recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload_view::<str>() {
                        println!(
                            "[{}]: Received message: {:?}",
                            "kafka -- consumer_order_task".green(),
                            payload.unwrap_or("")
                        );
    
                        // Send the received message directly to risk_sender
                        if risk_sender
                            .send(payload.unwrap_or("").to_string())
                            .await
                            .is_err()
                        {
                            eprintln!(
                                "[{}]: Failed to send order to risk management.",
                                "kafka -- consumer_order_task".red()
                            );
                            break;
                        }
                    } else {
                        println!(
                            "[{}]: Error while deserializing message",
                            "kafka -- consumer_order_task".red()
                        );
                    }
    
                    // Commit the message offset after processing
                    consumer
                        .commit_message(&message, CommitMode::Async)
                        .unwrap();
                }
                Err(e) => {
                    // Handle any errors while consuming
                    eprintln!(
                        "[{}]: Error while consuming: {:?}",
                        "kafka -- consumer_order_task".red(),
                        e
                    );
                    break;
                }
            }
        }
    }
       
}

// BaseProducer version
// pub fn kafka_send_stock(producer: Arc<Mutex<BaseProducer>>, receiver: Receiver<Vec<Stock>>, topic: String){

//     while let Ok(stocks) = receiver.recv() {
//         // Serialize Stock vector to JSON
//         let message = serde_json::to_string(&stocks).expect("Failed to serialize Stock objects");

//         println!("Sending to Kafka: {}", message);

//         // Send message to Kafka
//         let producer = producer.lock().expect("Failed to lock kafka producer");
//         let status = producer.send(BaseRecord::to(&topic).payload(&message).key("key"));

//         match status {
//             Ok(_) => println!("Message sent successfully to Kafka"),
//             Err(e) => eprintln!("Failed to send message to Kafka: {:?}", e),
//         }

//         // Flush producer to ensure delivery
//         producer.flush(Duration::from_secs(1));
//     }
// }