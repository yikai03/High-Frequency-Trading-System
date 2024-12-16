// stock.rs

use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use colored::Colorize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stock {
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapStock{
    // HashMap
    pub stocks: HashMap<String, f64>,
}

impl MapStock {
    pub fn new(sender: mpsc::Sender<Stock>) -> Self {
        // Initialize an empty HashMap
        let mut map_stock = MapStock {
            stocks: HashMap::new(),
        };

        // Populate the HashMap with stock data
        let stocks = vec![
            ("APPL", 100.00), 
            ("MSFT", 100.00), 
            ("GOOG", 100.00), 
            ("AMZN", 100.00), 
            ("TSLA", 100.00), 
            ("NVDA", 100.00), 
            ("META", 100.00), 
            ("ORCL", 100.00), 
            ("IBM", 100.00), 
            ("AMD", 100.00), 
            ("ADM", 100.00), 
            ("BG", 100.00), 
            ("FMC", 100.00), 
            ("CTVA", 100.00), 
            ("DE", 100.00), 
            ("MOS", 100.00), 
            ("AGCO", 100.00), 
            ("CF", 100.00), 
            ("CALM", 100.00), 
            ("SMG", 100.00), 
            ("XOM", 100.00), 
            ("CVX", 100.00), 
            ("BP", 100.00), 
            ("COP", 100.00), 
            ("TOT", 100.00), 
            ("HAL", 100.00), 
            ("SLB", 100.00), 
            ("PSX", 100.00), 
            ("VLO", 100.00), 
            ("OXY", 100.00), 
            ("JNJ", 100.00), 
            ("PFE", 100.00), 
            ("MRK", 100.00), 
            ("UNH", 100.00), 
            ("ABBV", 100.00), 
            ("AMGN", 100.00), 
            ("TMO", 100.00), 
            ("BMY", 100.00), 
            ("GILD", 100.00), 
            ("BIIB", 100.00), 
            ("JPM", 100.00), 
            ("BAC", 100.00), 
            ("WFC", 100.00), 
            ("GS", 100.00), 
            ("MS", 100.00), 
            ("C", 100.00), 
            ("USB", 100.00), 
            ("BK", 100.00), 
            ("TFC", 100.00), 
            ("AXP", 100.00), 
            ("PG", 100.00), 
            ("KO", 100.00), 
            ("PEP", 100.00), 
            ("UL", 100.00), 
            ("NKE", 100.00), 
            ("COST", 100.00), 
            ("MCD", 100.00), 
            ("WMT", 100.00), 
            ("SBUX", 100.00), 
            ("HD", 100.00),
        ];

        for (name, price) in stocks {
            map_stock.stocks.insert(name.to_string(), price);

        }
        let cloned_stock = map_stock.stocks.clone();
        tokio::spawn(async move {
            for (name, price) in cloned_stock {
                let name_clone = name.clone();
                let stock = Stock {
                    name: name.clone(),
                    price,
                };
                if sender.send(stock).await.is_err() {
                    eprintln!("[{}]: Failed to send stock: {}", "stock -- MapStock".red(), name_clone);
                }
            }
        });
        
        map_stock
    }

    pub fn get_stock_names(&self) -> Vec<String> {
        self.stocks.keys().cloned().collect()
    }

    pub fn read(&self, name: &str) -> Option<f64> {
        self.stocks.get(name).cloned()
    }
}

