extern crate scheduled_thread_pool;

use rand::Rng;
use std::{time::Duration,thread,sync::mpsc::channel};
use serde::{Deserialize,Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    pub name: String,
    pub stock_name: String,
    pub amount_unit: i32,
    pub request: bool,
}

impl Order {
    pub fn new(name: &str, stock_name: &str, amount_unit: i32, request: bool) -> Self {
        Self {
            name: name.to_string(),
            stock_name: stock_name.to_string(),
            amount_unit,
            request,
        }
    }

    pub fn get_stock_name(&self) -> &str {
        &self.stock_name
    }

    pub fn get_amount_unit(&self) -> i32 {
        self.amount_unit
    }

    pub fn is_request(&self) -> bool {
        self.request
    }
}

pub fn create_order(batch: &[String]) -> Result<Order, std::sync::mpsc::RecvError> {
    let (sender, receiver) = channel();
    let mut handles = vec![];

    for i in 1..6 {
        let batch_clone = batch.to_vec();
        let sender_clone = sender.clone();

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let rand_index = rng.gen_range(0..batch_clone.len());
            let random_value = rng.gen_range(1..=5);
            let client_name = format!("Client {}", i);
            let request = rand::random();

            let order = Order::new(&client_name, 
                &extract_stock_name(&batch_clone[rand_index]), 
                random_value, request);

            thread::sleep(Duration::from_secs(1));

            sender_clone.send(order).unwrap();

        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    if let Ok(order) = receiver.recv() {
        return Ok(order);
    }

    Err(std::sync::mpsc::RecvError)
}


pub fn extract_stock_name(message: &str) -> String {
    // Assuming the message format is consistent and always includes the stock name in double quotes
    if let Some(start_index) = message.find("name: \"") {
        if let Some(end_index) = message[start_index + 7..].find('"') {
            return message[start_index + 7..start_index + 7 + end_index].to_string();
        }
    }
    // Return an empty string if stock name cannot be extracted
    String::new()
}

