extern crate scheduled_thread_pool;
extern crate rand;
extern crate crossbeam_channel;

use std::sync::{Arc, Mutex};
use scheduled_thread_pool::ScheduledThreadPool;
use std::{time::Duration,vec};
use crossbeam_channel::unbounded;
use serde::{Deserialize,Serialize};
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Publish, Result, ExchangeType};
use crossbeam_channel::Sender;
use rand::thread_rng;
use crate::stock::rand::Rng;
use crate::client::Order;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stock {
    pub name: String,
    pub value: f64,
    pub unit: i32,
    pub current_unit: i32,
}

impl Stock {
    pub fn new(name: String, value: f64, unit: i32, current_unit: i32) -> Stock {
        Stock {
            name,
            value,
            unit,
            current_unit,
        }
    }
}

pub fn stocks(){
    
    let sched = ScheduledThreadPool::new(10);
    let (stock_s,stock_r) = unbounded();                              
    let (stock_sender, stock_receiver) = unbounded::<Order>();

    let stocks = vec![
        "TENAGA", "PCHEM", "IHH", "CDB", "PMETAL", "PETGAS", "MISC", "SIMEPLT", "NESTLE", "MAXIS",
        "AXIATA", "IOICORP", "TM", "KLK", "PPB", "PETDAG", "SIME", "SUNWAY", "BJFOOD", "GAMUDA", 
        "AEONCR","AIRASIA", "ALLIANZ", "AMMB", "ASTRO", "ATAIMS", "AXREIT", "BJTOTO", "BAUTO", "BAT", 
        "ARMADA","BURSA", "CMSB", "CARLSBG", "DSONIC", "DRBHCOM", "DPHARMA", "EKOVEST", "FGV", "F&N", 
        "FRONTKN","GDEX", "GENP", "GCB", "HEIM", "IGBREIT", "IJM", "INARI", "IOIPG", "KLCC", 
        "KOSSAN", "KPJ", "LHI", "LCTITAN", "MAGNUM", "MALAKOF", "MAHB", "MBSB", "MPI", "MRCB", 
        "MATRIX", "MEGAFIRS","MI", "MMC", "MRDIY", "MYEG", "PADINI", "PENTA", "QL", "SAPNRG",
    ];

    let shared_stock = Arc::new(Mutex::new(
        stocks.iter()
              .map(|name| {
                let unit = thread_rng().gen_range(100..300);
                Stock {
                  name: name.to_string(),
                  value: thread_rng().gen_range(50.0..1000.0),
                  unit: unit,
                  current_unit: unit,
                }
              })
              .collect::<Vec<Stock>>()
    ));

    let stock_s_clone = stock_s.clone();
    let stock_arc = shared_stock.clone();
    
    sched.execute_at_fixed_rate(
        Duration::from_secs(0),
        Duration::from_millis(300),
        move || {
            let mut stocks = stock_arc.lock().unwrap();
            while let Ok(order_received) = stock_receiver.try_recv() {  
                for stock in stocks.iter_mut() {            
                    if order_received.stock_name == stock.name {
                        let mut receive_current_unit = 0;   
                        if order_received.is_request() {
                            receive_current_unit = stock.current_unit - order_received.amount_unit;
                            stock_s_clone.send((1, stock.clone(),order_received.amount_unit,receive_current_unit)).unwrap();
                        } else {
                            receive_current_unit = stock.current_unit + order_received.amount_unit;
                            stock_s_clone.send((2, stock.clone(),order_received.amount_unit,receive_current_unit)).unwrap();
                        }
                    }
                }
            }
            for stock in stocks.iter_mut() {  
                if let Err(err) = broadcaster(&stock.clone()){
                    eprint!("Error broadcasting stock: {:?}",err);
                }   
            }
        }     
    );
 
    for _i in 1..=5 {
        let stock_arc = shared_stock.clone();
        let stock_r_clone = stock_r.clone();
        sched.execute_at_fixed_rate(
            Duration::from_secs(0),
            Duration::from_millis(500),
            move || {
                let (action, mut stock,difference,update_unit) = stock_r_clone.recv().unwrap();
                let inc: f64;
                match action {
                    1 => { // Increment
                        inc = match difference {
                            1 => 10.0,
                            2 => 20.0,
                            3 => 30.0,
                            4 => 40.0,
                            5 => 50.0,
                            _ => 0.0
                        };
                        let mut stocks = stock_arc.lock().unwrap();
                        for stock_item in stocks.iter_mut() {
                            if stock_item.name == stock.name {
                                println!("Before Value: {:.2}, Unit: {} ", stock_item.value, stock_item.current_unit);
                                stock_item.value += inc; 
                                stock_item.current_unit = update_unit;
                                println!("After Value: {:.2}, Unit: {} ", stock_item.value, stock_item.current_unit);
                                println!("📈 Incremented stock {} by {}", stock.name, inc);
                                println!();
                                if let Err(err) = broadcaster(&stock_item) {
                                    eprintln!("Error broadcasting stock: {:?}", err);
                                }
                            }
                        }
                    }
                    2 => { // Decrement
                        inc = match difference {
                            1 => 10.0,
                            2 => 20.0,
                            3 => 30.0,
                            4 => 40.0,
                            5 => 50.0,
                            _ => 0.0
                        };
                        let mut stocks = stock_arc.lock().unwrap();
                        for stock_item in stocks.iter_mut() {
                            if stock_item.name == stock.name {
                                println!("Before Value: {:.2}, Unit: {} ", stock_item.value, stock_item.current_unit);
                                stock_item.value -= inc; 
                                if stock_item.value < 50.0 {
                                    stock_item.value = 50.0;
                                }
                                stock_item.current_unit = update_unit;
                                println!("After Value: {:.2}, Unit: {} ", stock_item.value, stock_item.current_unit);
                                println!("📉 Decremented stock {} by {}", stock_item.name, inc);
                                println!();
                                if let Err(err) = broadcaster(&stock_item) {
                                    eprintln!("Error broadcasting stock: {:?}", err);
                                }
                            }
                        }
                    }
                    _ => {} //No action needed for other cases
                }
            }
        );
    }
   
    let _ = receive_order(stock_sender);
    
    loop{};
    
}

pub fn receive_order(sender: Sender<Order>) -> Result<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;

    let channel = connection.open_channel(None)?;

    let queue = channel.queue_declare("stock_market", QueueDeclareOptions::default())?;
 
    let consumer = queue.consume(ConsumerOptions::default())?;
 
    for (_, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&delivery.body);
                let parsed: Result<Order, serde_json::Error> = serde_json::from_str(&body);
                match parsed {
                    Ok(order) => {
                        let _ = sender.send(order);
                    },
                    Err(e) => println!("Error parsing JSON: {}", e),
                }
                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }
    connection.close()?;
    Ok(())
}


pub fn broadcaster(stock: &Stock) -> Result<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;

    let channel = connection.open_channel(None)?;

    let exchange = channel.exchange_declare(
        ExchangeType::Fanout,
        "fanout_exchange",
        Default::default(),
    )?;
    
    exchange.publish(Publish::new(format!("{:?}", stock).as_bytes(), ""))?;

    connection.close()
}

pub fn extract_stock_info(input: &str) -> Option<Stock> {
    let parts: Vec<&str> = input.split(", ")
        .map(|s| s.trim())
        .collect();

    if parts.len() != 4 {
        return None;
    }

    let name = parts[0].trim_start_matches("Stock { name: \"").trim_end_matches("\"");
    let value = parts[1].trim_start_matches("value: ").trim_end_matches(",");
    let unit = parts[2].trim_start_matches("unit: ").trim_end_matches(",");
    let current_unit = parts[3].trim_start_matches("current_unit: ").trim_end_matches(" }");

    let value: Result<f64, _> = value.parse();
    let unit: Result<i32, _> = unit.parse();
    let current_unit: Result<i32, _> = current_unit.parse();

    match (name, value, unit, current_unit) {
        (name, Ok(value), Ok(unit), Ok(current_unit)) => {
            Some(Stock {
                name: name.to_string(),
                value,
                unit,
                current_unit,
            })
        }
        _ => None,
    }
}











