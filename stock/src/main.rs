mod stock;
mod kafka;
mod risk;
mod order_dummy;
mod order_book;
mod log;

extern crate colored;

use tokio::sync::mpsc;
use tokio;
use colored::Colorize;
use std::sync::Arc;

use kafka::KafkaConfig;
use stock::MapStock;
use order_book::OrderBookHandler;
use risk::risk_task;


#[tokio::main]
async fn main() {
    // Connect to kafka
    let brokers = "localhost:9092";
    let produce_topic = "stock";
    // let consume_topic = "orders";
    let rejected_order_topic = "rejected_order";
    let group_id = "order_group";
    let completed_order_topic = "completed_order";

    let kafka_config = Arc::new(KafkaConfig::new(brokers, group_id));

    let (stock_sender, stock_receiver) = mpsc::channel(32);
    let (risk_sender, risk_receiver) = mpsc::channel(32);
    // let (order_sender, order_receiver) = mpsc::channel(32);
    let (rejected_order_sender, rejected_order_receiver) = mpsc::channel(32);
    let (order_book_sender, order_book_receiver) = mpsc::channel(32);
    let (completed_order_sender, completed_order_receiver) = mpsc::channel(32);


    // This thread will produce stock data to kafka
    let kafka_config_producer = Arc::clone(&kafka_config);
    tokio::spawn(async move {
        kafka_config_producer.producer_task(produce_topic, stock_receiver).await;
    });
    
    // This thread will consume order data from kafka
    let kafka_config_consumer = Arc::clone(&kafka_config);
    tokio::spawn(async move {
        kafka_config_consumer.consumer_order_task(risk_sender).await;
    });

    // // This thread will produce order data to kafka, which is dummy
    // let kafka_config_order_producer = Arc::clone(&kafka_config);
    // tokio::spawn(async move {
    //     kafka_config_order_producer.producer_task_for_order(consume_topic, order_receiver).await;
    // });

    // This thread will produce rejected order data to kafka
    let kafka_config_rejected_order_producer = Arc::clone(&kafka_config);
    tokio::spawn(async move {
        kafka_config_rejected_order_producer.producer_task_for_order(rejected_order_topic, rejected_order_receiver).await;
    });

    // This thread will produce completed order data to kafka
    let kafka_config_completed_order_producer = Arc::clone(&kafka_config);
    tokio::spawn(async move {
        kafka_config_completed_order_producer.producer_task_for_order(completed_order_topic, completed_order_receiver).await;
    });
    
    // Create a map of stock data
    let map_stock = MapStock::new(stock_sender.clone());
    println!("{}: {:?}","Stock data".blue(), map_stock.stocks);

    let order_book_handler = OrderBookHandler::new("order_book", map_stock.clone());

    // Start the risk management task to process messages
    tokio::spawn(async move {
        risk_task(risk_receiver, map_stock, rejected_order_sender, order_book_sender).await;
    });

    // // Start the order simulation task to send order data
    // tokio::spawn(async move {
    //     order_dummy::simulate_order(order_sender).await;
    // });

    tokio::spawn(async move {
        order_book_handler.add_to_order_book(order_book_receiver, completed_order_sender, stock_sender).await;
    });

    // Allow tasks to run indefinitely
    tokio::signal::ctrl_c().await.unwrap();
    println!("Application exiting...");
}

