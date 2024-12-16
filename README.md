# System Introduction

## Stock Simulation Software
This project simulates 60 stock prices and provides real-time updates to the frontend using Kafka and WebSocket. At its core, the system is built on a Rust-based backend, which powers essential features such as an order book, risk validation, and logsheet recording. These features ensure that stock simulations are accurate, efficient, and maintain a complete audit trail of transactions.

The frontend complements this system by displaying real-time stock price updates in an interactive table with percentage changes and visual cues, making it easy for users to analyze stock performance dynamically.

### **Features**

#### **Core Backend Features (Rust)**
- **Order Book Management**: Efficiently tracks buy and sell orders for stocks in real-time, enabling accurate price simulations and trade matching.
- **Risk Validation**: Implements safeguards to ensure simulated transactions meet predefined risk thresholds.
- **Logsheet Record**: Maintains a comprehensive log of all simulated trades and stock price updates for transparency and debugging.

#### **Kafka Integration**
- **Real-Time Messaging**: Sends stock price updates to a Kafka topic (`stocks`), ensuring low-latency delivery to connected clients.
- **Dynamic Updates**: Continuously pushes individual stock updates, reflecting price changes and fluctuations.

#### **Frontend Features (React)**
- **Sortable Stock Table**: Displays all 60 stocks with the latest price and percentage changes in descending order of price.
- **Dynamic Percentage Indicators**: Calculates and shows the percentage change for each stock in **green (increase)** or **red (decrease)** for better visibility.
- **WebSocket Integration**: Establishes a WebSocket connection to the backend for real-time updates directly from Kafka.

# System Setup
## Docker Setup
1. cd './Assignment'
2. docker compose up -d

> [!IMPORTANT]
If you docker gives you an issue, run below CLI to manual create a topic

### Docker CLI 
1. cd './Assignment'

2. docker compose up -d

3. docker exec -it kafka bash

4. kafka-topics.sh --list --bootstrap-server localhost:9092

5. kafka-topics.sh --describe --topic stock --bootstrap-server localhost:9092

6. kafka-topics.sh --create --topic stock --bootstrap-server localhost:9092 --replication-factor 1 --partitions 1

7. exit

8. docker exec -it kafka kafka-topics.sh --list --bootstrap-server localhost:9092


9. docker exec -it kafka kafka-console-producer.sh --topic stock --bootstrap-server localhost:9092

	a. write something to send


10. open a new terminal

11. docker exec -it kafka kafka-console-consumer.sh --topic stock --bootstrap-server localhost:9092 --from-beginning
	
    a. should see all the things produced in

## Simulation of Stock Market - GUI (Backend)
1. Ensure docker is up
2. Open New Terminal
2. cd "Assignment/simulation/backend"
3. npm install
4. node kafkaconsumer.js

## Simulation of Stock Market - GUI (Frontend)
1. Ensure docker is up
2. Open New Terminal
3. cd "Assignment/simulation/frontend"
4. npm install
5. npm start

## Simulation of Stock Market (Rust)
1. Ensure docker, backend, and frontend is up
2. Open New Terminal
3. cd "Assignment/stock"
4. cargo run
