use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::order_dummy::OrderRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logger {
    file_path: String,
}

impl Logger {
    // Create a new logger with the specified file path
    pub fn new(file_path: &str) -> Self {
        OpenOptions::new()
        .create(true)  // Create the file if it doesn't exist
        .write(true)   // Open in write mode
        .truncate(true) // Clear existing content
        .open(file_path)
        .expect("Failed to create or clear the log file");

        Self {
            file_path: file_path.to_string(),
        }
    }

    // Log a message to the log file
    pub fn log(&self, message: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        writeln!(file, "{}", message)?;
        Ok(())
    }

    // Add a structured log entry for completed orders and stock updates
    pub fn add(
        &self,
        completed_best_order: &OrderRequest,
        completed_order: &OrderRequest,
        stock_name: &str,
        stock_price: f64,
    ) -> std::io::Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let timestamp = now.as_secs();
        let log_message = format!(
            "[{}] Executed Order: [Best Order: {:?}] [Market Order: {:?}] | \n\nUpdated Stock: [Name: {}, Price: {:.2}]\n------------------",
            timestamp,
            completed_best_order,
            completed_order,
            stock_name,
            stock_price
        );
        self.log(&log_message)?;
        Ok(())
    }
}
