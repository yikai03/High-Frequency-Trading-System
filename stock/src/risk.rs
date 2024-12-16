// risk.rs

use tokio::sync::mpsc;
use colored::Colorize;

use crate::stock::MapStock;
use crate::order_dummy::{OrderRequest, OrderStatus};

pub async fn risk_task(
    mut order_receiver: mpsc::Receiver<Vec<String>>, 
    stock: MapStock, 
    rejected_order_sender: mpsc::Sender<OrderRequest>,
    order_book_sender: mpsc::Sender<OrderRequest>,
    ) {
    while let Some(message) = order_receiver.recv().await {
        println!("[{}]:  Order received: {:?}", "Risk management".green(), message);

        let order: OrderRequest = serde_json::from_str(&message[0]).unwrap();

        let stock_price = stock.read(&order.stock_symbol.clone()).unwrap();

        let mut rejected = false;
        
        // 1. Order price should within currect price +- 20%
        if order.price < stock_price * 0.8 || order.price > stock_price * 1.2 {
            println!("[{}]: Order price is out of range", "Risk management".red());
            rejected = true;
        }

        // 2. Order quantity should be less than 1000
        if order.quantity > 1000 {
            println!("[{}]: Order quantity is too large", "Risk management".red());
            rejected = true;
        }

        if rejected{
            let rejected_order = OrderRequest {
                broker_id: order.broker_id,
                client_id: order.client_id,
                order_id: order.order_id.clone(),
                stock_symbol: order.stock_symbol.clone(),
                order_type: order.order_type.clone(),
                order_action: order.order_action.clone(),
                price: order.price,
                quantity: order.quantity,
                status: OrderStatus::Rejected,
            };

            rejected_order_sender.send(rejected_order).await.unwrap();
        } else {
            order_book_sender.send(order).await.unwrap();
        }
    }
}
