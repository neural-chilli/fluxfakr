# FluxFakr

## Overview
**FluxFakr** is a **modular, high-performance data stream generator** built in **Rust**. It allows users to simulate **realistic synthetic data flows** for testing, analytics, and AI-driven applications.

With its **pluggable architecture**, FluxFakr lets you define and swap generator modules effortlessly, making it highly customizable for different simulation needs.

---

## Features
- **Modular Design**: Easily swap or extend generator modules.
- **High-Performance Simulation**: Configurable message rates with real-time streaming.
- **Multiple Output Formats**: Supports **JSON streaming** and **CSV dumps**.
- **Configurable Data Flows**: Define parameters like message rate, drift, and volatility.
- **Extensible Architecture**: Implement new data generators using a simple Rust trait.

---

## Installation

### Prerequisites
- **Rust (Edition 2021)** and **Cargo** installed.
- Basic familiarity with command-line usage.

### Steps
Clone the repository and build the project in release mode:
```bash
git clone https://github.com/neural-chilli/fluxfakr.git
cd fluxfakr
cargo build --release
```

---

## Usage

Run FluxFakr with your desired parameters. For example, to simulate **market data** at **100 messages per second** for **5 instruments**, use:

```bash
./target/release/fluxfakr --module market-data --mps 100 --variants 5 \
--broker localhost:9092 --topic market-data
```

For **LangChain-generated data**, specify a prompt-based simulation:

```bash
./target/release/fluxfakr --module langchain-data --prompt "Generate customer transactions" --repeat 50
```

---

## Architecture

FluxFakr follows a **modular architecture** where each generator module implements a common `Generator` trait. This allows seamless integration of new data types without modifying the core engine.

### Generator Trait

```rust
pub trait Generator {
fn generate(&mut self) -> String;
fn dump(&self) -> String;
}
```

### Data Flow

1. **Input**: CLI parameters define the simulation (e.g., message rate, variants).
2. **Processing**: The selected generator produces synthetic data.
3. **Output**: JSON messages are streamed, and the internal state can be dumped to CSV.

---

## Extending FluxFakr

To add a new generator module:

1. Implement the `Generator` trait in a new Rust module under `src/generators/`.
2. Register the module in `src/generators/mod.rs`.
3. Compile and run tests to ensure functionality.

Example implementation:

```rust
pub struct CustomGenerator;

impl Generator for CustomGenerator {
fn generate(&mut self) -> String {
json!({ "event": "custom", "value": 42 }).to_string()
}

    fn dump(&self) -> String {
        "custom,data".to_string()
    }
}
```

---

## Roadmap

### Near-Term Goals
- Expand **Market Data Generator** with additional configurations.
- Integrate **LLM-based synthetic data** in the LangChain generator.
- Improve **documentation** with tutorials and examples.

### Mid-Term Goals
- **Performance Enhancements**: Optimize throughput and latency.
- **Web UI** for live configuration and monitoring.
- Support for **custom JSON templates** in generator modules.

### Long-Term Goals
- Foster **community contributions** and engagement.
- Integrate with **data visualization tools**.
- Develop **plugins** for streaming platforms and analytics dashboards.

---

## Contributing

Contributions are welcome! To get started:

1. **Fork & Clone** the repository:
   ```bash
   git clone https://github.com/neural-chilli/fluxfakr.git
   cd fluxfakr
   ```
2. **Build & Run Tests**:
   ```bash
   cargo build && cargo test
   ```
3. Submit a **pull request** with clear commit messages.

---

## License

FluxFakr is open-source software, licensed under **MIT**.

---

**Happy coding with FluxFakr! 🚀**
```
