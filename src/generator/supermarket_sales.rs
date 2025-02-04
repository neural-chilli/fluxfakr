use chrono::Utc;
use fake::faker::address::en::{CityName, StateAbbr};
use fake::Fake;
use once_cell::sync::Lazy;
use rand::Rng;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

/// Represents a product in a hierarchical catalog.
#[derive(Debug, Serialize, Clone)]
pub struct Product {
    pub product_name: String,
    pub category: String,
    pub subcategory: String,
    pub unit_price: f64,
}

/// Represents store details.
#[derive(Debug, Serialize, Clone)]
pub struct Store {
    pub town: String,
    pub state: String,
    pub country: String,
}

/// Represents customer demographic data.
#[derive(Debug, Serialize, Clone)]
pub struct Customer {
    pub age: u8,
    pub income_band: String,
}

/// Represents a sale for a single product.
#[derive(Debug, Serialize)]
pub struct SaleMessage {
    pub transaction_id: String,
    pub basket_id: String,
    pub timestamp: u64,
    pub store: Store,
    pub customer: Customer,
    pub product: Product,
    pub quantity: u32,
    pub total_price: f64,
}

/// A product hierarchy that simulates a supermarket product catalog.
/// Each tuple is (Category, list of (Subcategory, list of Product Names)).
static PRODUCT_HIERARCHY: &[(&str, &[(&str, &[&str])])] = &[
    (
        "Food",
        &[
            (
                "Canned Goods",
                &["Tomato Soup", "Baked Beans", "Corn", "Peaches"],
            ),
            ("Bakery", &["Bread", "Croissant", "Bagel", "Muffin"]),
            ("Deli", &["Ham", "Turkey", "Cheese", "Salami"]),
            ("Produce", &["Apples", "Bananas", "Carrots", "Lettuce"]),
            (
                "Frozen",
                &[
                    "Ice Cream",
                    "Frozen Pizza",
                    "Frozen Vegetables",
                    "Frozen Dinners",
                ],
            ),
        ],
    ),
    (
        "Beauty",
        &[
            (
                "Skincare",
                &["Moisturizer", "Cleanser", "Sunscreen", "Serum"],
            ),
            ("Makeup", &["Lipstick", "Mascara", "Foundation", "Eyeliner"]),
            (
                "Haircare",
                &["Shampoo", "Conditioner", "Hair Gel", "Hair Spray"],
            ),
            ("Fragrances", &["Perfume", "Cologne", "Body Mist"]),
        ],
    ),
    (
        "Healthcare",
        &[
            (
                "Pharmacy",
                &[
                    "Pain Reliever",
                    "Cough Syrup",
                    "Antibiotics",
                    "Antihistamines",
                ],
            ),
            (
                "Vitamins",
                &["Multivitamin", "Vitamin C", "Vitamin D", "Calcium"],
            ),
            (
                "Medical Supplies",
                &["Bandages", "Antiseptic", "Thermometer", "Gloves"],
            ),
        ],
    ),
    (
        "Cleaning Products",
        &[
            (
                "Household Cleaners",
                &["All-Purpose Cleaner", "Glass Cleaner", "Disinfectant"],
            ),
            (
                "Laundry",
                &["Detergent", "Fabric Softener", "Stain Remover"],
            ),
            ("Dishwashing", &["Dish Soap", "Dishwasher Detergent"]),
        ],
    ),
    (
        "Pets",
        &[
            (
                "Pet Food",
                &["Dog Food", "Cat Food", "Bird Seed", "Fish Flakes"],
            ),
            ("Toys", &["Chew Toy", "Catnip Toy", "Interactive Toy"]),
            ("Grooming", &["Shampoo", "Comb", "Nail Clippers"]),
        ],
    ),
    (
        "Clothing",
        &[
            ("Men", &["T-Shirt", "Jeans", "Jacket", "Sneakers"]),
            ("Women", &["Dress", "Blouse", "Skirt", "Heels"]),
            ("Children", &["Kids T-Shirt", "Kids Jeans", "Kids Jacket"]),
            ("Accessories", &["Hat", "Scarf", "Belt", "Sunglasses"]),
        ],
    ),
];

/// A global cache for product prices keyed by (category, product_name).
static PRICE_CACHE: Lazy<Mutex<HashMap<(String, String), f64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Compute a deterministic raw price (in dollars) for a given product name and category.
/// The price range is determined by the category:
/// - Food: $1–$10
/// - Beauty: $5–$30
/// - Healthcare: $3–$25
/// - Cleaning Products: $2–$15
/// - Pets: $3–$20
/// - Clothing: $5–$50
fn compute_price(category: &str, product_name: &str) -> f64 {
    let hash: u32 = product_name
        .bytes()
        .fold(0, |acc, b| acc.wrapping_add(b as u32));
    let (min, max) = match category {
        "Food" => (1.0, 10.0),
        "Beauty" => (5.0, 30.0),
        "Healthcare" => (3.0, 25.0),
        "Cleaning Products" => (2.0, 15.0),
        "Pets" => (3.0, 20.0),
        "Clothing" => (5.0, 50.0),
        _ => (1.0, 20.0),
    };
    let range = max - min;
    let scaled = (hash % 1000) as f64 / 1000.0;
    min + scaled * range
}

/// Round the price to the nearest value ending in .49 or .99.
/// It computes two candidates: base + 0.49 and base + 0.99, then chooses the closer one.
fn round_price(price: f64) -> f64 {
    let base = price.floor();
    let candidate1 = base + 0.49;
    let candidate2 = base + 0.99;
    if (price - candidate1).abs() <= (price - candidate2).abs() {
        candidate1
    } else {
        candidate2
    }
}

/// Retrieve the product price from cache (or compute, round, and cache it if not already computed).
fn get_product_price(category: &str, product_name: &str) -> f64 {
    let key = (category.to_string(), product_name.to_string());
    {
        let cache = PRICE_CACHE.lock().unwrap();
        if let Some(&price) = cache.get(&key) {
            return price;
        }
    }
    let raw_price = compute_price(category, product_name);
    let final_price = round_price(raw_price);
    let mut cache = PRICE_CACHE.lock().unwrap();
    cache.insert(key, final_price);
    final_price
}

/// Generate a product using the product hierarchy.
/// The unit price is computed deterministically and then rounded, using the cache.
fn generate_product() -> Product {
    let mut rng = rand::rng();
    let (category, subcategories) = PRODUCT_HIERARCHY[rng.random_range(0..PRODUCT_HIERARCHY.len())];
    let (subcategory, products) = subcategories[rng.random_range(0..subcategories.len())];
    let product_name = products[rng.random_range(0..products.len())];
    let unit_price = get_product_price(category, product_name);
    Product {
        product_name: product_name.to_string(),
        category: category.to_string(),
        subcategory: subcategory.to_string(),
        unit_price,
    }
}

/// Generate store details using fake data, limited to America.
fn generate_store() -> Store {
    let town: String = CityName().fake();
    let state: String = StateAbbr().fake();
    let country = "USA".to_string();
    Store {
        town,
        state,
        country,
    }
}

/// Generate customer demographic data using fake data.
fn generate_customer() -> Customer {
    let mut rng = rand::rng();
    let age = rng.random_range(18..80);
    let income_bands = ["Low", "Medium", "High"];
    let income_band = income_bands[rng.random_range(0..income_bands.len())].to_string();
    Customer {
        age,
        income_band,
    }
}

/// Generate a sale message for a single product sale.
/// Each sale message includes details of the store, customer, and product sold.
fn generate_sale_message(
    transaction_id: &str,
    basket_id: &str,
    store: &Store,
    customer: &Customer,
) -> SaleMessage {
    let product = generate_product();
    let mut rng = rand::rng();
    let quantity = rng.random_range(1..5); // Quantity between 1 and 4.
    let total_price = product.unit_price * quantity as f64;
    let timestamp: u64 = Utc::now().timestamp() as u64;
    SaleMessage {
        transaction_id: transaction_id.to_string(),
        basket_id: basket_id.to_string(),
        timestamp,
        store: store.clone(),
        customer: customer.clone(),
        product,
        quantity,
        total_price,
    }
}

/// A Basket represents a shopping basket (a single transaction) that will produce multiple sale messages.
#[derive(Debug)]
struct Basket {
    transaction_id: String,
    basket_id: String,
    store: Store,
    customer: Customer,
    total_items: usize,
    items_generated: usize,
}

/// SalesGenerator is our generator for FluxFakr. It produces one sale message per call.
/// When a basket is exhausted, it automatically creates a new basket.
pub struct SalesGenerator {
    current_basket: Option<Basket>,
}

impl SalesGenerator {
    pub fn new() -> Self {
        SalesGenerator {
            current_basket: None,
        }
    }

    /// Initialize a new basket with the given number of items.
    pub fn init_basket(&mut self, basket_size: u32) {
        let mut rng = rand::rng();
        let transaction_id = format!("TXN-{:08}", rng.random_range(0..100000000));
        let basket_id = format!("BASKET-{:04}", rng.random_range(0..10000));
        let store = generate_store();
        let customer = generate_customer();
        self.current_basket = Some(Basket {
            transaction_id,
            basket_id,
            store,
            customer,
            total_items: basket_size as usize,
            items_generated: 0,
        });
    }
}

impl crate::Generator for SalesGenerator {
    fn generate(&mut self) -> String {
        // If there is no basket or if the current basket is exhausted, initialize a new basket.
        if self.current_basket.is_none()
            || self.current_basket.as_ref().unwrap().items_generated
                >= self.current_basket.as_ref().unwrap().total_items
        {
            let mut rng = rand::rng();
            let basket_size = rng.random_range(5..16);
            self.init_basket(basket_size);
        }

        if let Some(ref mut basket) = self.current_basket {
            basket.items_generated += 1;
            let sale = generate_sale_message(
                &basket.transaction_id,
                &basket.basket_id,
                &basket.store,
                &basket.customer,
            );
            serde_json::to_string(&sale).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        }
    }

    fn dump(&self) -> String {
        if let Some(ref basket) = self.current_basket {
            format!(
                "Basket Summary: transaction_id: {}, basket_id: {}, items_generated: {}, total_items: {}",
                basket.transaction_id, basket.basket_id, basket.items_generated, basket.total_items
            )
        } else {
            "No basket data available.".to_string()
        }
    }
}
