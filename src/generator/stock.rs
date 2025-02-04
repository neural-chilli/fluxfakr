use rand::Rng;
use rand_distr::{Distribution, StandardNormal};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a simulated stock instrument with realistic market data.
#[derive(Debug)]
pub struct Instrument {
    pub id: String,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub volume: u64,
}

/// A Market Data Generator that simulates realistic stock price updates.
pub struct StockDataGenerator {
    instruments: Vec<Instrument>,
}

impl StockDataGenerator {
    /// Creates a new MarketDataGenerator with the specified number of variants.
    pub fn new(variants: usize) -> Self {
        let mut rng = rand::rng();
        let instruments = (0..variants)
            .map(|i| {
                let price = rng.random_range(100.0..200.0);
                // Initialize bid/ask as a small spread around the price.
                let spread = price * rng.random_range(0.001..0.002);
                Instrument {
                    id: format!("STK{}", i),
                    price,
                    bid: price - spread,
                    ask: price + spread,
                    volume: 0,
                }
            })
            .collect();
        StockDataGenerator { instruments }
    }
}

impl crate::Generator for StockDataGenerator {

    fn generate(&mut self) -> String {
        let mut rng = rand::rng();

        if self.instruments.is_empty() {
            return "{}".to_string();
        }

        // Randomly select an instrument to update.
        let idx = rng.random_range(0..self.instruments.len());
        let instrument = &mut self.instruments[idx];

        // --- Price Update using Geometric Brownian Motion ---
        //
        // Geometric Brownian Motion:
        //   S(t+dt) = S(t) * exp((mu - 0.5 * sigma^2)*dt + sigma * sqrt(dt) * epsilon)
        //
        // We'll use a small time increment dt, a slight drift (mu) and volatility (sigma).
        let dt: f64 = 1.0 / 252.0; // assume one trading day step (or one iteration) in yearly terms
        let mu = 0.0001;      // drift term (very small positive drift)
        let sigma = 0.01;     // volatility (1% per time step)
        let epsilon: f64 = StandardNormal.sample(&mut rng);
        let change_factor = ((mu - 0.5 * sigma * sigma) * dt + sigma * dt.sqrt() * epsilon).exp();
        instrument.price = (instrument.price * change_factor).max(0.01);

        // --- Bid/Ask Spread Update ---
        //
        // The spread is a small fraction of the price that can vary with market conditions.
        // We add a bit of randomness on top of a base spread fraction.
        let base_spread_fraction = 0.001; // base 0.1%
        let extra_spread: f64 = rng.random_range(0.0..0.001); // additional randomness up to 0.1%
        let spread_fraction = base_spread_fraction + extra_spread;
        let spread = instrument.price * spread_fraction;
        instrument.bid = instrument.price - spread;
        instrument.ask = instrument.price + spread;

        // --- Volume Update ---
        //
        // We simulate trade volume as a base volume plus some random fluctuation.
        let base_volume = 1000;
        let volume_variation = rng.random_range(0..500);
        let trade_volume = base_volume + volume_variation;
        instrument.volume += trade_volume as u64;

        // --- Timestamp ---
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Build a JSON message containing the updated instrument data.
        let message = json!({
            "instrument": instrument.id,
            "price": instrument.price,
            "bid": instrument.bid,
            "ask": instrument.ask,
            "volume": instrument.volume,
            "timestamp": now,
        });
        message.to_string()
    }

    fn dump(&self) -> String {
        // Build a CSV header with the relevant fields.
        let mut csv = String::from("id,price,bid,ask,volume\n");
        for instrument in &self.instruments {
            csv.push_str(&format!(
                "{},{:.2},{:.2},{:.2},{}\n",
                instrument.id,
                instrument.price,
                instrument.bid,
                instrument.ask,
                instrument.volume
            ));
        }
        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Generator;
    use serde_json::Value;

    #[test]
    fn test_generate_market_data() {
        let mut generator = StockDataGenerator::new(5);
        let msg_str = generator.generate();
        // Parse the JSON message to ensure it is valid.
        let msg: Value = serde_json::from_str(&msg_str).unwrap();
        assert!(msg.get("instrument").is_some());
        assert!(msg.get("price").is_some());
        assert!(msg.get("bid").is_some());
        assert!(msg.get("ask").is_some());
        assert!(msg.get("volume").is_some());
        assert!(msg.get("timestamp").is_some());
    }

    #[test]
    fn test_dump_market_data() {
        let generator = StockDataGenerator::new(3);
        let csv = generator.dump();
        let lines: Vec<&str> = csv.lines().collect();
        // Expect a header plus one line per instrument.
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "id,price,bid,ask,volume");
        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            assert_eq!(parts.len(), 5);
            parts[1].parse::<f64>().unwrap();
            parts[2].parse::<f64>().unwrap();
            parts[3].parse::<f64>().unwrap();
            parts[4].parse::<u64>().unwrap();
        }
    }
}
