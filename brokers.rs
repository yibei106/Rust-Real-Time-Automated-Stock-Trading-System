use crate::client::{create_order, Order};
use crate::stock::extract_stock_info;

use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Exchange, Publish, Result, ExchangeType};
use serde_json::{self};

pub fn broker(input: &str) -> Result<()>{
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;

    let channel = connection.open_channel(None)?;

    let exchange = channel.exchange_declare(
        ExchangeType::Fanout,
        "fanout_exchange",
        Default::default(),
    )?;

    let queue = channel.queue_declare("", QueueDeclareOptions::default())?;

    queue.bind(&exchange, "", Default::default())?;

    let consumer = queue.consume(ConsumerOptions::default())?;
    
    let mut batch: Vec<String> = Vec::with_capacity(70);
    let mut orders: Vec<Order> = Vec::with_capacity(10);
    let mut sell_list: Vec<Order> = Vec::with_capacity(10);
    let mut stocks_list: Vec<Order> = Vec::new();

    for (_, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                batch.push(body.to_string());

                if batch.len() == 70 {
                    match create_order(&batch) {
                        Ok(order) => {
                            if order.is_request() {
                                println!("🛒 {} requests to buy {} for {} units",order.name, order.stock_name, order.amount_unit);
                                println!();
                                orders.push(order);
                            } else {
                                println!("💰 {} requests to sell {} for {} units", order.name, order.stock_name, order.amount_unit);
                                println!();
                                sell_list.push(order);
                            }
                        }
                        Err(err) => {
                            eprintln!("Error creating order: {:?}", err);
                        }
                    }
                    batch.clear();
                }
                    consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }


        let mut index_to_remove = None;
        for (order_index, order_item) in orders.iter().enumerate() {
            for batch_item in &batch {
                if let Some(stock) = extract_stock_info(batch_item) {
                    if order_item.get_stock_name() == stock.name {
                        if stock.current_unit - stock.unit > 3 || stock.current_unit < 50  {
                            println!("🚫 {} rejected the order for buying {} !", input, stock.name);
                            println!();
                            index_to_remove = Some(order_index);
                        } else {
                            println!("💵 {} buying the stock {}.", input, stock.name);
                            println!();
                            
                            stocks_list.push(order_item.clone());
                            
                            let _ = send_order(&order_item);

                            index_to_remove = Some(order_index);
                        }
                    }
                } else {
                    println!("Failed to extract unit and current_unit from the message.");
                }
            }
            if let Some(index) = index_to_remove {
                orders.remove(index);
                break;
            }
        }
        

        let mut index_to_sell = None;
        for (sell_index, sell_item) in sell_list.iter().enumerate() {
            for batch_item in &batch {
                if let Some(stock) = extract_stock_info(batch_item) {
                    if sell_item.get_stock_name() == stock.name {
                        println!("💸 {} selling the stock {}.", input, stock.name);
                        println!();

                        let _ = send_order(&sell_item);

                        index_to_sell = Some(sell_index);
                    }
                } else {
                    println!("Failed to extract unit and current_unit from the message.");
                }
            }
            if let Some(index) = index_to_sell {
                sell_list.remove(index);
                break;
            }
        }

        let mut index_sellinventory = None;
        for (inventory_index, inventory_item) in stocks_list.iter().enumerate() {
            for batch_item in &batch {
                if let Some(stock) = extract_stock_info(batch_item) {
                    if inventory_item.get_stock_name() == stock.name {
                        if stock.current_unit - stock.unit > 2 {
                            println!("📣 {} is oversold!", stock.name);
                            println!();
                            println!("📦 {} is selling stock {} from inventory.", input, stock.name);
                            println!();

                            let _ = send_order(&inventory_item);

                            index_sellinventory = Some(inventory_index);

                        }
                    }
                } else {
                    println!("Failed to extract unit and current_unit from the message.");
                }
            }
            if let Some(index) = index_sellinventory{
                stocks_list.remove(index);
                break;
            }
        }
    }

    connection.close()?;

    Ok(())
}


fn send_order(order_info: &Order) -> Result<()> {
    let mut connection = Connection::insecure_open(
        "amqp://guest:guest@localhost:5672")?;

    let channel = connection.open_channel(None)?;

    let exchange = Exchange::direct(&channel);

    let stock_json = serde_json::to_string(order_info).unwrap();
    exchange.publish(Publish::new(stock_json.as_bytes(),"stock_market"))?;

    connection.close()?;

    Ok(())
}