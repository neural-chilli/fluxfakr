mod generator;

use clap::Parser;
use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use crate::generator::Generator;
use crate::generator::stock::StockDataGenerator;
use crate::generator::supermarket_sales::SalesGenerator;

/// FluxFakr: A modular data stream generator.
#[derive(Parser, Debug)]
#[command(name = "FluxFakr", about = "A modular data stream generator.")]
struct Cli {
    /// Generator module to use (e.g., market, other)
    #[arg(long)]
    module: String,

    /// Messages per second (must be > 0)
    #[arg(long)]
    mps: u32,

    /// Number of unique simulated entities (variants)
    #[arg(long)]
    variants: u32,

    /// Broker address (optional; e.g., Kafka broker)
    #[arg(long)]
    broker: Option<String>,

    /// Output topic name (optional)
    #[arg(long)]
    topic: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Display optional settings.
    if let Some(broker) = &cli.broker {
        println!("Broker set to: {}", broker);
    }
    if let Some(topic) = &cli.topic {
        println!("Topic set to: {}", topic);
    }

    // Validate messages-per-second.
    if cli.mps == 0 {
        eprintln!("Error: mps (messages per second) must be greater than 0");
        std::process::exit(1);
    }

    // Generator manager section
    let mut generator: Box<dyn Generator> = match cli.module.as_str() {
        "stock" => Box::new(StockDataGenerator::new(cli.variants as usize)),
        "supermarket" => Box::new(SalesGenerator::new()),
        _ => {
            eprintln!("Unknown module: {}", cli.module);
            std::process::exit(1);
        }
    };

    // Calculate sleep duration between messages.
    let sleep_duration = Duration::from_secs_f64(1.0 / cli.mps as f64);

    // Set up Kafka producer if both broker and topic are provided.
    let mut kafka_producer: Option<BaseProducer> = None;
    if let (Some(broker), Some(topic)) = (cli.broker.clone(), cli.topic.clone()) {
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", &broker)
            .create()
            .expect("Kafka producer creation error");
        kafka_producer = Some(producer);
        println!("Kafka producer initialized for topic: {}", topic);
    }

    // Create a flag to indicate whether the simulation is running.
    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            println!("\nInterrupt received! Shutting down gracefully...");
            running.store(false, Ordering::SeqCst);
        })
            .expect("Error setting Ctrl-C handler");
    }

    println!("Simulation started. Press Ctrl+C to stop.");

    // Simulation loop: continue until interrupted.
    while running.load(Ordering::SeqCst) {
        let message = generator.generate();

        // If a Kafka producer is available, send the message to Kafka.
        if let Some(producer) = kafka_producer.as_mut() {

            let topic = cli.topic.as_ref().unwrap();
            let record = BaseRecord::to(topic).payload(&message).key("");
            if let Err((e, _)) = producer.send(record) {
                eprintln!("Failed to send message to Kafka: {}", e);
            }
            // Poll to handle any delivery callbacks.
            producer.poll(Duration::from_millis(0));
        }
        println!("{}", message);
        io::stdout().flush().unwrap();
        thread::sleep(sleep_duration);
    }

    // Flush any remaining Kafka messages.
    if let Some(producer) = kafka_producer.as_mut() {
        producer.flush(Duration::from_secs(5)).unwrap();
    }

    // On exit, dump the generator's internal state
    println!("\n--- Generator Internal State Dump ---");
    println!("{}", generator.dump());
}
