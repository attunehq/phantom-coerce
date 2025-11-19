/// Example demonstrating cloned coercion for message format conversion
///
/// This shows how to use phantom types to track message formats, and clone
/// messages to generic format types for logging or archival purposes.
use phantom_coerce::Coerce;
use std::marker::PhantomData;

// Format markers
#[derive(Clone)]
struct Json;
#[derive(Clone)]
struct Xml;
#[derive(Clone)]
struct Protobuf;
#[derive(Clone)]
struct AnyFormat; // Generic format (subsumes all specific formats)

// Priority markers
#[derive(Clone)]
struct High;
#[derive(Clone)]
struct Normal;
#[derive(Clone)]
struct Low;
#[derive(Clone)]
struct AnyPriority; // Generic priority

#[derive(Coerce, Clone, Debug)]
#[coerce(
    cloned_from = "Message<Json | Xml | Protobuf, High>",
    cloned_to = "Message<AnyFormat, High>"
)]
#[coerce(
    cloned_from = "Message<Json, High | Normal | Low>",
    cloned_to = "Message<Json, AnyPriority>"
)]
#[coerce(
    cloned_from = "Message<Json | Xml | Protobuf, High | Normal | Low>",
    cloned_to = "Message<AnyFormat, AnyPriority>"
)]
struct Message<Format, Priority> {
    format: PhantomData<Format>,
    priority: PhantomData<Priority>,
    id: u64,
    content: String,
    metadata: Vec<String>,
    timestamp: u64,
}

impl Message<Json, High> {
    fn new_json_high(id: u64, content: &str) -> Self {
        Self {
            format: PhantomData,
            priority: PhantomData,
            id,
            content: content.to_string(),
            metadata: vec!["format:json".to_string(), "priority:high".to_string()],
            timestamp: current_timestamp(),
        }
    }
}

impl Message<Xml, Normal> {
    fn new_xml_normal(id: u64, content: &str) -> Self {
        Self {
            format: PhantomData,
            priority: PhantomData,
            id,
            content: content.to_string(),
            metadata: vec!["format:xml".to_string(), "priority:normal".to_string()],
            timestamp: current_timestamp(),
        }
    }
}

impl Message<Protobuf, Low> {
    fn new_protobuf_low(id: u64, content: &str) -> Self {
        Self {
            format: PhantomData,
            priority: PhantomData,
            id,
            content: content.to_string(),
            metadata: vec!["format:protobuf".to_string(), "priority:low".to_string()],
            timestamp: current_timestamp(),
        }
    }
}

impl<Format, Priority> Message<Format, Priority> {
    fn content(&self) -> &str {
        &self.content
    }

    fn id(&self) -> u64 {
        self.id
    }
}

// Archive function that accepts any format/priority
fn archive_message(msg: Message<AnyFormat, AnyPriority>) {
    println!(
        "  📦 Archived message {} (timestamp: {}, metadata: {})",
        msg.id,
        msg.timestamp,
        msg.metadata.len()
    );
}

// High-priority message logger (only for high priority, any format)
fn log_high_priority<Format>(msg: &Message<Format, High>) {
    println!(
        "  ⚠️  HIGH PRIORITY message {}: {}",
        msg.id(),
        msg.content()
    );
}

// JSON-specific processor (only JSON, any priority)
fn process_json_message<Priority>(msg: &Message<Json, Priority>) {
    println!("  🔧 Processing JSON message {}: parsing...", msg.id());
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn main() {
    println!("=== Message Format Handling with Cloned Coercion ===\n");

    // Create messages with different formats and priorities
    let json_high = Message::<Json, High>::new_json_high(1, r#"{"event": "user_login"}"#);

    let xml_normal = Message::<Xml, Normal>::new_xml_normal(2, "<event>data_sync</event>");

    let proto_low = Message::<Protobuf, Low>::new_protobuf_low(3, "binary_data_here");

    println!("--- Processing Messages ---");

    // Process JSON message with format-specific handler
    // The original message remains available after this
    process_json_message(&json_high);

    // Log high-priority messages
    // Uses cloned coercion to treat Json message as AnyFormat for logging
    log_high_priority(&json_high.to_coerced());

    // The original json_high is still usable!
    println!(
        "  ✓ Original JSON message still accessible: id={}",
        json_high.id()
    );

    println!();

    println!("--- Archival System ---");
    println!("Archiving messages (requires generic format and priority):");

    // Clone and coerce to fully generic type for archival
    // Note: We clone here so originals remain available
    archive_message(json_high.to_coerced());
    archive_message(xml_normal.to_coerced());
    archive_message(proto_low.to_coerced());

    println!();

    println!("--- Messages Still Available ---");
    println!("After archival, we can still process the originals:");
    println!("  • JSON message content: {}", json_high.content());
    println!("  • XML message content: {}", xml_normal.content());
    println!("  • Protobuf message content: {}", proto_low.content());

    println!();

    println!("--- Partial Coercion ---");
    println!("We can also coerce just one parameter:");

    // Coerce only the priority to generic, keeping JSON format
    let json_any_priority: Message<Json, AnyPriority> = json_high.to_coerced();
    println!(
        "  • JSON message with any priority: id={}",
        json_any_priority.id()
    );

    println!("\n=== Key Takeaway ===");
    println!("Cloned coercion allows non-destructive conversion of strongly-typed");
    println!("messages to generic types. Original messages remain available for");
    println!("format-specific processing while generic copies can be archived or logged.");
}
