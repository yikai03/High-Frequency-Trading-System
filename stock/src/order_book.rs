use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tokio::sync::mpsc;
use tokio;
use colored::Colorize;

use crate::{order_dummy::{OrderRequest, OrderAction, OrderType, OrderStatus}, stock::{MapStock, Stock}, log::Logger};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderBookHandler {
    pub name: String,
    pub logger: Logger,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderBook{
    pub buy_orders: Vec<OrderRequest>,
    pub sell_orders: Vec<OrderRequest>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            buy_orders: Vec::new(),
            sell_orders: Vec::new(),
        }
    }
}

impl OrderBookHandler {
    pub fn new(name: &str, stock_data: MapStock) -> Self {
        let order_book = OrderBookHandler {
            name: name.to_string(),
            logger: Logger::new("order_book.log"),
        };

        // get all the stock name in stock_data and create a file storing limit order for each stock
        let stock_names = stock_data.get_stock_names();

        // Define a base directory to store the order book files
        let base_dir = "order_books";

        // Ensure the directory exists, then clean it
        if Path::new(base_dir).exists() {
            // Remove all files in the directory
            if let Err(e) = fs::remove_dir_all(base_dir) {
                eprintln!("[{}]: Failed to clean up order book directory: {}", "order_book -- OrderBookHandler".red() ,e);
            }
        }

        // Create the directory and files
        fs::create_dir_all(base_dir).unwrap(); // Ensure the directory exists

        // Create a JSON file for each stock
        for stock_name in stock_names {
            let file_path = format!("{}/{}.json", base_dir, stock_name);

            // Create a new file if it doesn't exist
            if !std::path::Path::new(&file_path).exists() {
                let initial_order_book = OrderBook::new();
                if let Err(e) = OrderBookHandler::save_to_file(&initial_order_book, &file_path) {
                    eprintln!("[{}]: Failed to create JSON file for {}: {}", "order_book -- OrderBookHandler".red(), stock_name, e);
                }
            }
        }

        order_book
    }

    pub async fn add_to_order_book(&self, mut order_book_receiver: mpsc::Receiver<OrderRequest>, mut completed_order_sender: mpsc::Sender<OrderRequest>, mut stock_updater_sender: mpsc::Sender<Stock>) {
        while let Some(mut order) = order_book_receiver.recv().await {
            let file_path = format!("order_books/{}.json", order.stock_symbol);
            let mut order_book = OrderBookHandler::load_from_file(&file_path).unwrap();
            match order.order_type {
                OrderType::Market => {
                    println!("[{}]: {} order received: {:?}", "order_book -- OrderBook".green(), "Market".bold().on_bright_purple(), order);

                    // Attempt to execute the market order immediately
                    let execution_success = match order.order_action {
                        OrderAction::Buy => execute_market_order(&mut order, &mut order_book.sell_orders, &mut completed_order_sender, &mut stock_updater_sender, &self.logger).await,
                        OrderAction::Sell => execute_market_order(&mut order, &mut order_book.buy_orders, &mut completed_order_sender, &mut stock_updater_sender, &self.logger).await,
                        // OrderAction::Cancel => false, // Market orders cannot be canceled directly
                    };

                    if !execution_success {
                        // Convert to limit order and add to the order book
                        println!("{}", "Market order complete execution failed, with remain quantity. Converting to limit order.".italic().yellow());
                        order.order_type = OrderType::Limit;
                        match order.order_action {
                            OrderAction::Buy => {
                                order_book.buy_orders.push(order);
                                order_book.buy_orders.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                            }
                            OrderAction::Sell => {
                                order_book.sell_orders.push(order);
                                order_book.sell_orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                            }
                            // _ => {}
                        }
                    }
                },
                OrderType::Limit => {
                    println!("[{}]: {} order received: {:?}", "order_book -- OrderBook".green(), "Limit".bold().on_bright_purple(), order);

                    match order.order_action {
                        OrderAction::Buy => {
                            order_book.buy_orders.push(order);
                            order_book.buy_orders.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                        },
                        OrderAction::Sell => {
                            order_book.sell_orders.push(order);
                            order_book.sell_orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                        },
                        // OrderAction::Cancel => {
                        //     order_book.buy_orders.retain(|x| x.order_id != order.order_id);
                        //     order_book.sell_orders.retain(|x| x.order_id != order.order_id);
                        // }
                    }
                }
            }
            if let Err(e) = OrderBookHandler::save_to_file(&order_book, &file_path) {
                eprintln!("[{}]: Failed to save order book to file: {}", "order_book -- OrderBook".red(), e);
            }
        }
    }


    /// ----Support Function----
    /// Save an `OrderBook` struct to a file in JSON format
    pub fn save_to_file(order_book: &OrderBook, file_path: &str) -> std::io::Result<()> {
        let json_data = serde_json::to_string_pretty(order_book).unwrap(); // Serialize the `OrderBook` to JSON
        let mut file = File::create(file_path)?; // Create or overwrite the file
        file.write_all(json_data.as_bytes())?; // Write the JSON data to the file
        Ok(())
    }
    
    /// Load an `OrderBook` struct from a JSON file
    pub fn load_from_file(file_path: &str) -> std::io::Result<OrderBook> {
        let json_data = std::fs::read_to_string(file_path)?; // Read the file content
        let order_book: OrderBook = serde_json::from_str(&json_data).unwrap(); // Deserialize JSON into `OrderBook`
        Ok(order_book)
    }
}

/// Executes a market order by matching it with the opposing order book.
/// Returns `true` if the order is fully executed, `false` otherwise.
async fn execute_market_order(
    order: &mut OrderRequest, 
    opposing_orders: &mut Vec<OrderRequest>, 
    completed_order_sender: &mut mpsc::Sender<OrderRequest>,
    stock_updater_sender: &mut mpsc::Sender<Stock>,
    logger: &Logger,
    ) -> bool {
    let mut remaining_quantity = order.quantity;

    // three situtation
    // 1. current order quantity is more than best order quantity
        // 1.1 while loop match current with best order[0]
            // 1.1.1 execute the best order[0]
            // 1.1.2 send complete message to kafka
            // 1.1.3 send latest price to stock updater
            // 1.1.4 remove the best order[0]
            // 1.1.5 look back the three situation
    // 2. current order quantity is less than best order quantity
        // 2.1 while loop match current with best order[0]
            // 2.1.1 execute the current order
            // 2.1.2 send complete message to kafka
            // 2.1.3 send latest price to stock updater
            // 2.1.4 update the best order quantity
            // 2.1.5 look back the three situation
    // 3. current order quantity is equal to best order quantity
        // 3.1 execute the current order
        // 3.2 send complete message to kafka
        // 3.3 send latest price to stock updater
        // 3.4 remove the best order[0]
        // 3.5 look back the three situation

    while remaining_quantity > 0 && !opposing_orders.is_empty() {
        let best_order = &mut opposing_orders[0];

        if remaining_quantity >= best_order.quantity {
            // Fully consume the best order
            remaining_quantity -= best_order.quantity;
            // Generate complete order structure for both side, current order and best order
            let completed_best_order = OrderRequest { // existing limit order
                broker_id: best_order.broker_id,
                client_id: best_order.client_id,
                order_id: best_order.order_id.clone(),
                stock_symbol: best_order.stock_symbol.clone(),
                order_type: best_order.order_type.clone(),
                order_action: best_order.order_action.clone(),
                price: best_order.price,
                quantity: best_order.quantity,
                status: OrderStatus::Completed,
            };
            let completed_order = OrderRequest { // current market order
                broker_id: order.broker_id,
                client_id: order.client_id,
                order_id: order.order_id.clone(),
                stock_symbol: order.stock_symbol.clone(),
                order_type: order.order_type.clone(),
                order_action: order.order_action.clone(),
                price: best_order.price, //Current market order take the best order price
                quantity: best_order.quantity,
                status: OrderStatus::Completed,
            };

            println!("[{}]: {} {:?} with {:?}", "order_book -- OrderBook".green(), "Executed".bold().on_bright_cyan(), completed_order, completed_best_order);

            // Send the completed orders to Kafka
            completed_order_sender.send(completed_best_order.clone()).await.unwrap();
            completed_order_sender.send(completed_order.clone()).await.unwrap();

            // Send the latest price to the stock updater
            let updated_stock = Stock {
                name: best_order.stock_symbol.clone(),
                price: best_order.price,
            };
            stock_updater_sender.send(updated_stock.clone()).await.unwrap();


            logger.add(
                &completed_best_order,
                &completed_order,
                &updated_stock.name,
                updated_stock.price,
            ).unwrap();

            opposing_orders.remove(0); // Remove the matched order

        } else {
            // Partially consume the best order (limit order)
            best_order.quantity -= remaining_quantity;

            // Generate complete order structure for both side, current order and best order
            let completed_best_order = OrderRequest { // existing limit order
                broker_id: best_order.broker_id,
                client_id: best_order.client_id,
                order_id: best_order.order_id.clone(),
                stock_symbol: best_order.stock_symbol.clone(),
                order_type: best_order.order_type.clone(),
                order_action: best_order.order_action.clone(),
                price: best_order.price,
                quantity: remaining_quantity,
                status: OrderStatus::Completed,
            };

            let completed_order = OrderRequest { // current market order
                broker_id: order.broker_id,
                client_id: order.client_id,
                order_id: order.order_id.clone(),
                stock_symbol: order.stock_symbol.clone(),
                order_type: order.order_type.clone(),
                order_action: order.order_action.clone(),
                price: best_order.price,
                quantity: remaining_quantity,
                status: OrderStatus::Completed,
            };

            println!("[{}]: {} {:?} with {:?}", "order_book -- OrderBook".green(), "Executed".bold().on_bright_cyan(), completed_order, completed_best_order);

            // Send the completed orders to Kafka
            completed_order_sender.send(completed_best_order.clone()).await.unwrap();
            completed_order_sender.send(completed_order.clone()).await.unwrap();

            // Send the latest price to the stock updater
            let updated_stock = Stock {
                name: best_order.stock_symbol.clone(),
                price: best_order.price,
            };
            stock_updater_sender.send(updated_stock.clone()).await.unwrap();

            logger.add(
                &completed_best_order,
                &completed_order,
                &updated_stock.name,
                updated_stock.price,
            ).unwrap();

            remaining_quantity = 0;
        }
    }
    // Update the remaining quantity in the market order
    order.quantity = remaining_quantity;

    // Return true if fully executed
    remaining_quantity == 0
}
