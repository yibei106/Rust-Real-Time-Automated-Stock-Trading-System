## Rust-Real-Time-Automated-Stock-Trading-System

## Project Overview
This project simulates a stock market system where brokers interact with clients and the stock market to facilitate buy and sell orders. It showcases how concurrent programming techniques, message-passing mechanisms, and real-time decision-making can be modeled in a dynamic stock trading environment.

The primary focus of the project is to simulate real-time stock adjustments based on orders received from clients through brokers. The system leverages multithreading, message exchanges, and synchronization mechanisms to ensure data consistency and safe concurrent operations.

## Concept Diagram
![image](https://github.com/user-attachments/assets/9301b3d5-529c-49f5-940d-02ed52f42198)

## Features
- **Stock Market Simulation**: Brokers receive stock information and relay it to clients. Clients place buy or sell orders, and brokers evaluate these orders based on market conditions.
- **Thread-Safe Concurrency**: The project uses Rust’s `Arc` (Atomic Reference Counting) and `Mutex` to manage concurrent access to shared data, ensuring thread safety and avoiding race conditions.
- **Crossbeam Channels & Message Exchanges**: Internal stock market communications utilize Crossbeam channels. Brokers and the stock market communicate using exchange mechanisms (fanout and direct exchanges).
- **Stock Management**: Each stock has a constant **unit** value and a **current unit**, which fluctuates based on buy or sell orders. Specific conditions determine whether orders are accepted or rejected.
- **Multithreading**: Threads are used to simulate clients and brokers in parallel, while a thread pool is used to handle scheduled tasks like stock value updates.
- **Message Broadcasting**: The system uses a fanout exchange to broadcast stock information to brokers and a direct exchange to route orders between brokers and the stock market.
- **Serialization**: Stocks are serialized into JSON for transmission and deserialized upon receipt for processing.
- **Benchmarking**: The system is benchmarked using **Criterion**, a Rust benchmarking tool.

## Architecture
- **Brokers**: Receive stock information from the stock market and pass it to clients. Brokers also collect buy/sell orders from clients and relay them back to the stock market for processing.
- **Clients**: Represented by threads, clients generate random buy or sell orders for stocks. Each client processes orders independently.
- **Stock Market**: Adjusts stock values based on buy or sell orders received from brokers.
- **Thread Pool**: A pool of 10 threads is used for handling scheduled tasks such as periodic stock value updates. Five additional threads are spawned to manage stock adjustment tasks based on buy/sell orders.
- **Crossbeam Channels**: Used for internal communication within the stock market. Two unbounded channels are used: 
  1. One for communication between the selector thread and the increment/decrement handling thread.
  2. Another for passing Order objects between the exchange consumer and the thread selector.
- **Fanout Exchange**: Used to broadcast stock information to brokers.
- **Direct Exchange**: Routes orders between brokers and the stock market, ensuring accurate message delivery based on routing keys.

## Stock Processing Workflow
1. **Broadcasting**: Initially, the stock list is shared with all brokers via a fanout exchange.
2. **Order Placement**: Clients send buy or sell orders to brokers. These orders are categorized into buy or sell vectors for further processing.
3. **Order Evaluation**: 
   - For **buy orders**, the broker checks if the difference between the `current unit` and `unit` exceeds a certain threshold (e.g., 3). If so, the order is rejected.
   - For **sell orders**, no conditions are imposed, and the stock value is decremented.
4. **Stock Adjustment**: The stock market adjusts the stock value based on accepted orders:
   - **Increment**: For buy orders, the stock value increases, capped at a maximum of 50 units.
   - **Decrement**: For sell orders, the stock value decreases, updating the `current unit`.
5. **Message Passing**: The updated stock information is sent to the brokers, and the stock market processes further orders based on broker messages via the direct exchange.

