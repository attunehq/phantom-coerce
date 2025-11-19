//! Example demonstrating owned coercion for request validation states
//!
//! This shows how to use phantom types to track validation state, and coerce
//! validated requests to a generic "any status" type for storage or API boundaries.

use phantom_coerce::Coerce;
use std::marker::PhantomData;

// Validation state markers
struct Unvalidated;
struct HeadersValidated;
struct FullyValidated;
struct AnyStatus; // Generic status that can represent any validation state

// HTTP method markers
struct Get;
struct Post;
struct AnyMethod; // Generic method

#[derive(Coerce, Debug)]
#[coerce(
    owned_from = "Request<HeadersValidated | FullyValidated, Get>",
    owned_to = "Request<AnyStatus, Get>"
)]
#[coerce(
    owned_from = "Request<FullyValidated, Get | Post>",
    owned_to = "Request<FullyValidated, AnyMethod>"
)]
#[coerce(
    owned_from = "Request<HeadersValidated | FullyValidated, Get | Post>",
    owned_to = "Request<AnyStatus, AnyMethod>"
)]
struct Request<Status, Method> {
    status: PhantomData<Status>,
    method: PhantomData<Method>,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl Request<Unvalidated, Get> {
    fn new_get(url: &str) -> Self {
        Self {
            status: PhantomData,
            method: PhantomData,
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }
}

impl Request<Unvalidated, Post> {
    fn new_post(url: &str, body: &[u8]) -> Self {
        Self {
            status: PhantomData,
            method: PhantomData,
            url: url.to_string(),
            headers: Vec::new(),
            body: Some(body.to_vec()),
        }
    }
}

impl<Method> Request<Unvalidated, Method> {
    fn add_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    fn validate_headers(self) -> Result<Request<HeadersValidated, Method>, String> {
        if self.headers.iter().any(|(k, _)| k == "Authorization") {
            Ok(Request {
                status: PhantomData,
                method: self.method,
                url: self.url,
                headers: self.headers,
                body: self.body,
            })
        } else {
            Err("Missing Authorization header".to_string())
        }
    }
}

impl<Method> Request<HeadersValidated, Method> {
    fn validate_url(self) -> Result<Request<FullyValidated, Method>, String> {
        if self.url.starts_with("https://") {
            Ok(Request {
                status: PhantomData,
                method: self.method,
                url: self.url,
                headers: self.headers,
                body: self.body,
            })
        } else {
            Err("URL must use HTTPS".to_string())
        }
    }
}

// Functions that work with generic types
fn store_request(req: Request<AnyStatus, AnyMethod>) {
    println!("Storing request: {} (any status, any method)", req.url);
}

fn log_validated_request<Method>(req: Request<FullyValidated, Method>) {
    println!("Logging validated request: {}", req.url);
}

fn main() {
    println!("=== Request Validation with Owned Coercion ===\n");

    // Create and validate a GET request
    println!("--- GET Request ---");
    let get_req = Request::<Unvalidated, Get>::new_get("https://api.example.com/users")
        .add_header("Authorization", "Bearer token123")
        .add_header("Accept", "application/json");

    match get_req.validate_headers() {
        Ok(req) => match req.validate_url() {
            Ok(validated) => {
                println!("✓ GET request fully validated");

                // We can log it with its full validation status
                log_validated_request(validated);

                // Or coerce to generic method for storage (consumes the request)
                // Uncommenting this would prevent using validated after this point
                // let generic: Request<FullyValidated, AnyMethod> = validated.into_coerced();
                // store_request(generic.into_coerced());
            }
            Err(e) => println!("✗ URL validation failed: {}", e),
        },
        Err(e) => println!("✗ Header validation failed: {}", e),
    }

    println!();

    // Create and validate a POST request
    println!("--- POST Request ---");
    let post_req =
        Request::<Unvalidated, Post>::new_post("https://api.example.com/users", b"user data")
            .add_header("Authorization", "Bearer token456")
            .add_header("Content-Type", "application/json");

    match post_req.validate_headers() {
        Ok(req) => match req.validate_url() {
            Ok(validated) => {
                println!("✓ POST request fully validated");

                // Coerce to generic status and method for storage
                // This allows us to store both GET and POST requests in the same collection
                let generic: Request<AnyStatus, AnyMethod> = validated.into_coerced();
                store_request(generic);
            }
            Err(e) => println!("✗ URL validation failed: {}", e),
        },
        Err(e) => println!("✗ Header validation failed: {}", e),
    }

    println!();

    // Example of failing validation
    println!("--- Invalid Request ---");
    let invalid_req = Request::<Unvalidated, Get>::new_get("http://insecure.example.com")
        .add_header("Authorization", "Bearer token789");

    match invalid_req.validate_headers() {
        Ok(req) => match req.validate_url() {
            Ok(_) => println!("✓ Request validated (unexpected)"),
            Err(e) => println!("✗ Validation failed as expected: {}", e),
        },
        Err(e) => println!("✗ Header validation failed: {}", e),
    }

    println!("\n=== Key Takeaway ===");
    println!("Owned coercion allows consuming strongly-typed requests and converting");
    println!("them to generic types for storage, while maintaining type safety during");
    println!("the validation pipeline.");
}
