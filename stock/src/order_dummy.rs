use std::vec;
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng; 
use colored::Colorize;


#[derive(Serialize, Debug, Deserialize, Clone)]
pub enum OrderStatus {
    Pending,
    Completed,
    Rejected,
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub enum OrderAction {
    Buy,
    Sell,
    // Cancel,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct OrderRequest {
    pub broker_id: u64,
    pub client_id: u64,
    pub order_id: String,
    pub stock_symbol: String,
    pub order_type: OrderType,
    pub order_action: OrderAction,
    pub price: f64,
    pub quantity: usize,
    pub status: OrderStatus,
}

impl OrderRequest {
    pub fn new(broker_id: u64, client_id: u64, order_id: &str, stock_symbol: &str, order_type: OrderType, order_action: OrderAction, price: f64, quantity: usize, status: OrderStatus) -> Self {
        OrderRequest {
            broker_id,
            client_id,
            order_id: order_id.to_string(),
            stock_symbol: stock_symbol.to_string(),
            order_type,
            order_action,
            price,
            quantity,
            status,
        }
    }
}

pub async fn simulate_order(sender: mpsc::Sender<OrderRequest>) {  
    // Initialize a seedable RNG that is `Send`
    let mut rng = StdRng::from_entropy();

    let stock_symbols = vec!["APPL", "MSFT", "GOOG", "AMZN", "TSLA"];
    let order_types = vec![OrderType::Market, OrderType::Limit];
    let order_actions = vec![OrderAction::Buy, OrderAction::Sell];

    loop {
        let order_action = order_actions[rng.gen_range(0..order_actions.len())].clone(); // Random order_action
        let order_type = order_types[rng.gen_range(0..order_types.len())].clone(); // Random order_type
        let new_order = OrderRequest::new(
            rng.gen_range(1..=10), // Random broker_id
            rng.gen_range(1..=10), // Random client_id
            &format!("order{}", rng.gen_range(6..=100)), // Random order_id
            stock_symbols[rng.gen_range(0..stock_symbols.len())], // Random stock_symbol
            order_type.clone(), // Random order_type
            order_action.clone(), // Random order_action
            price_maker(order_action, order_type), // Random price based on order_action
            rng.gen_range(1..=10), // Random quantity
            OrderStatus::Pending, // Status
        );

        sleep(Duration::from_secs(1)).await;

        if sender.send(new_order.clone()).await.is_err() {
            eprintln!("Failed to send stock data.");
            break;
        }

        println!("[{}]: Sent order data: {:?}", "order_dummy -- OrderSimulated".green(), new_order);
    }
}

fn price_maker(action: OrderAction, market_or_limit: OrderType) -> f64 {
    let price = match market_or_limit{
        OrderType::Market => {
            100.00
        }

        OrderType::Limit => {
            match action {
                OrderAction::Buy => {
                    rand::thread_rng().gen_range(79.0..=99.0)
                }
                OrderAction::Sell => {
                    rand::thread_rng().gen_range(101.0..=121.0)
                }
            }
        }
    };

    price
}