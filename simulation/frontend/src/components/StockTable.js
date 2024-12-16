import React, { useState, useEffect } from 'react';

const StockTable = () => {
    const [stocks, setStocks] = useState({}); // Store stock data as a key-value pair: {name: {price, percentage}}

    useEffect(() => {
        const ws = new WebSocket('ws://localhost:8081'); // Connect to WebSocket server

        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);

            setStocks((prev) => {
                const previousStock = prev[data.name]; // Get previous stock data
                const previousPrice = previousStock?.price || data.price; // Use the same price if it’s the first update
                const percentageChange = ((data.price - previousPrice) / previousPrice).toFixed(4); // Calculate percentage

                return {
                    ...prev,
                    [data.name]: { price: data.price, percentage: percentageChange }, // Update the specific stock
                };
            });
        };

        return () => ws.close(); // Cleanup WebSocket connection
    }, []);

    // Convert stocks object to an array, sort by price in descending order
    const sortedStocks = Object.entries(stocks)
        .sort(([, a], [, b]) => b.price - a.price); // Sort stocks by price

    return (
        <table>
            <thead>
                <tr>
                    <th>Stock Name</th>
                    <th>Price</th>
                    <th>Change (%)</th>
                </tr>
            </thead>
            <tbody>
                {sortedStocks.map(([name, { price, percentage }]) => (
                    <tr key={name}>
                        <td>{name}</td>
                        <td>{price.toFixed(2)}</td>
                        <td style={{ color: percentage > 0 ? 'green' : 'red' }}>
                            {percentage > 0 ? '+' : ''}{(percentage * 100).toFixed(2)}%
                        </td>
                    </tr>
                ))}
            </tbody>
        </table>
    );
};

export default StockTable;
