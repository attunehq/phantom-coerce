/// Example demonstrating heterogeneous collections with coercion
///
/// This shows a key use case: storing items with different specific marker types
/// in the same collection by coercing them to a common generic type.
use phantom_coerce::Coerce;
use std::marker::PhantomData;

// Source types (where data comes from)
struct Database;
struct Api;
struct Cache;
struct FileSystem;
struct AnySource; // Generic source

// Processing state types
struct Raw;
struct Validated;
struct Enriched;
struct AnyState; // Generic state

#[derive(Coerce)]
#[coerce(borrowed = "DataItem<AnySource, Raw>")]
#[coerce(borrowed = "DataItem<Database, AnyState>")]
#[coerce(borrowed = "DataItem<AnySource, AnyState>")]
struct DataItem<Source, State> {
    source: PhantomData<Source>,
    state: PhantomData<State>,
    id: String,
    payload: String,
    size_bytes: usize,
}

impl DataItem<Database, Raw> {
    fn from_database(id: String, payload: String) -> Self {
        Self {
            source: PhantomData,
            state: PhantomData,
            id,
            size_bytes: payload.len(),
            payload,
        }
    }
}

impl DataItem<Api, Raw> {
    fn from_api(id: String, payload: String) -> Self {
        Self {
            source: PhantomData,
            state: PhantomData,
            id,
            size_bytes: payload.len(),
            payload,
        }
    }
}

impl DataItem<Cache, Validated> {
    fn from_cache(id: String, payload: String) -> Self {
        Self {
            source: PhantomData,
            state: PhantomData,
            id,
            size_bytes: payload.len(),
            payload,
        }
    }
}

impl DataItem<FileSystem, Enriched> {
    fn from_file(id: String, payload: String) -> Self {
        Self {
            source: PhantomData,
            state: PhantomData,
            id,
            size_bytes: payload.len(),
            payload,
        }
    }
}

impl<Source, State> DataItem<Source, State> {
    fn id(&self) -> &str {
        &self.id
    }

    fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    fn payload(&self) -> &str {
        &self.payload
    }
}

// Functions that work with generic collections
fn report_total_size(items: &[&DataItem<AnySource, AnyState>]) {
    let total: usize = items.iter().map(|item| item.size_bytes()).sum();
    println!("Total size: {} bytes across {} items", total, items.len());
}

fn list_all_ids(items: &[&DataItem<AnySource, AnyState>]) {
    println!("Item IDs:");
    for item in items {
        println!("  • {}", item.id());
    }
}

fn main() {
    println!("=== Heterogeneous Collections with Coercion ===\n");

    // Create items from different sources with different states
    let db_item = DataItem::<Database, Raw>::from_database("user_123".to_string(), "user data from DB".to_string());

    let api_item = DataItem::<Api, Raw>::from_api("order_456".to_string(), "order data from API".to_string());

    let cache_item =
        DataItem::<Cache, Validated>::from_cache("session_789".to_string(), "validated session data".to_string());

    let file_item =
        DataItem::<FileSystem, Enriched>::from_file("config_000".to_string(), "enriched config data".to_string());

    println!("--- Created Items with Specific Types ---");
    println!("✓ Database item (Raw): {}", db_item.id());
    println!("✓ API item (Raw): {}", api_item.id());
    println!("✓ Cache item (Validated): {}", cache_item.id());
    println!("✓ File item (Enriched): {}", file_item.id());

    println!("\n--- Creating Heterogeneous Collection ---");
    println!("Coercing all items to generic source and state...");

    // Coerce all items to the same generic type so they can be stored together
    let generic_items: Vec<&DataItem<AnySource, AnyState>> = vec![
        db_item.coerce(),
        api_item.coerce(),
        cache_item.coerce(),
        file_item.coerce(),
    ];

    println!("✓ Stored {} items in homogeneous collection", generic_items.len());

    println!("\n--- Processing Generic Collection ---");
    list_all_ids(&generic_items);
    println!();
    report_total_size(&generic_items);

    println!("\n--- Filtering by Source (Partial Coercion) ---");
    println!("We can also create collections that preserve some type information:");

    // Create a collection of only raw items (any source, but specifically Raw state)
    let raw_items: Vec<&DataItem<AnySource, Raw>> = vec![
        db_item.coerce(), // Database -> AnySource (keeping Raw)
        api_item.coerce(), // Api -> AnySource (keeping Raw)
        // cache_item is Validated, can't include it here
        // file_item is Enriched, can't include it here
    ];

    println!("Raw items collection:");
    for item in &raw_items {
        println!("  • {} ({} bytes)", item.id(), item.size_bytes());
    }

    println!("\n--- Another Collection: Database Items (Any State) ---");
    // Only database items, but any processing state
    let db_items: Vec<&DataItem<Database, AnyState>> = vec![
        db_item.coerce(), // Keep Database source, generalize state
        // api_item is from Api, can't include it
        // cache_item is from Cache, can't include it
        // file_item is from FileSystem, can't include it
    ];

    println!("Database items:");
    for item in &db_items {
        println!("  • {} ({} bytes)", item.id(), item.size_bytes());
    }

    println!("\n=== Key Takeaway ===");
    println!("Coercion enables heterogeneous collections by converting specific types");
    println!("to a common generic type. You can choose which type parameters to");
    println!("generalize and which to keep specific, giving you fine-grained control");
    println!("over type safety vs. flexibility.");
}
