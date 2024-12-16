const { Kafka } = require('kafkajs');
const WebSocket = require('ws');

const kafka = new Kafka({
    clientId: 'stock-simulator',
    brokers: ['localhost:9092'], // Replace with your Kafka broker(s)
});

const consumer = kafka.consumer({ groupId: 'stock-group' });

// WebSocket server for pushing updates to the frontend
const wss = new WebSocket.Server({ port: 8081 });

const run = async () => {
    await consumer.connect();
    await consumer.subscribe({ topic: 'stock', fromBeginning: false });

    consumer.run({
        eachMessage: async ({ topic, partition, message }) => {
            const stockUpdate = message.value.toString();
            console.log(`Received message: ${stockUpdate}`);

            // Broadcast the message to all connected clients
            wss.clients.forEach((client) => {
                if (client.readyState === WebSocket.OPEN) {
                    client.send(stockUpdate);
                }
            });
        },
    });
};

run().catch(console.error);
