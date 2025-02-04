pub mod market;
pub mod supermarket_sales;

pub trait Generator {
    /// Generate a JSON data message
    fn generate(&mut self) -> String;
    /// Dump the internal state
    fn dump(&self) -> String;
}

