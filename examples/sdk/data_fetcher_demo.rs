//! Comprehensive demo of data fetching capabilities
//!
//! This example demonstrates:
//! - Fetching data from HTTP APIs
//! - Reading files from the filesystem
//! - Parsing JSON data
//! - Error handling
//! - Both builder pattern and quick functions

use periplon_sdk::{DataFetcher, HttpMethod, HttpRequest};
use serde::{Deserialize, Serialize};
use std::io::Write;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiRequest {
    query: String,
    limit: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Data Fetcher Demo ===\n");

    // Example 1: Basic API fetching
    demo_basic_api_fetch().await?;

    // Example 2: POST request with JSON
    demo_api_post_json().await?;

    // Example 3: Advanced HTTP request with headers
    demo_advanced_http().await?;

    // Example 4: File operations
    demo_file_operations().await?;

    // Example 5: JSON file operations
    demo_json_files().await?;

    // Example 6: Error handling
    demo_error_handling().await?;

    // Example 7: Quick convenience functions
    demo_quick_functions().await?;

    println!("\n=== All demos completed successfully! ===");

    Ok(())
}

async fn demo_basic_api_fetch() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic API Fetch (GET request)");
    println!("   Fetching from mock API...");

    let fetcher = DataFetcher::new();
    let response = fetcher.get("https://api.example.com/users").await?;

    println!("   Status: {}", response.status);
    println!("   Success: {}", response.is_success());
    println!("   Body: {}", response.text());

    if let Ok(json) = response.json_value() {
        println!("   Parsed JSON: {}", serde_json::to_string_pretty(&json)?);
    }

    println!();
    Ok(())
}

async fn demo_api_post_json() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. POST Request with JSON Body");

    let request_data = ApiRequest {
        query: "rust programming".to_string(),
        limit: 10,
    };

    let fetcher = DataFetcher::new();
    let response = fetcher
        .post_json("https://api.example.com/search", &request_data)
        .await?;

    println!("   POST Status: {}", response.status);
    println!("   Response: {}", response.text());
    println!();

    Ok(())
}

async fn demo_advanced_http() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Advanced HTTP Request with Custom Headers");

    let request = HttpRequest::new("https://api.example.com/protected")
        .method(HttpMethod::Get)
        .header("Authorization", "Bearer secret-token-123")
        .header("X-Custom-Header", "custom-value")
        .timeout(60);

    let fetcher = DataFetcher::new()
        .user_agent("DataFetcherDemo/1.0")
        .default_header("X-App-Version", "1.0.0");

    let response = fetcher.fetch_http(request).await?;

    println!("   Status: {}", response.status);
    println!("   Headers: {:?}", response.headers);
    println!();

    Ok(())
}

async fn demo_file_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. File Operations");

    // Create a temporary test file
    let test_file = "/tmp/data_fetcher_test.txt";
    let mut file = std::fs::File::create(test_file)?;
    writeln!(file, "Line 1: Hello from file!")?;
    writeln!(file, "Line 2: This is a test file.")?;
    writeln!(file, "Line 3: Data fetching is awesome!")?;

    let fetcher = DataFetcher::new();

    // Check if file exists
    println!("   File exists: {}", fetcher.file_exists(test_file).await);

    // Get file metadata
    let metadata = fetcher.file_metadata(test_file).await?;
    println!("   File size: {} bytes", metadata.size);
    println!("   Is file: {}", metadata.is_file);
    println!("   Read-only: {}", metadata.read_only);

    // Read entire file
    let content = fetcher.read_text_file(test_file).await?;
    println!("   File content:\n{}", content);

    // Read file line by line
    let lines = fetcher.read_lines(test_file).await?;
    println!("   Total lines: {}", lines.len());
    for (i, line) in lines.iter().enumerate() {
        println!("     Line {}: {}", i + 1, line);
    }

    // Read binary
    let bytes = fetcher.read_binary_file(test_file).await?;
    println!("   Binary size: {} bytes", bytes.len());

    // Cleanup
    std::fs::remove_file(test_file)?;
    println!();

    Ok(())
}

async fn demo_json_files() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. JSON File Operations");

    // Create test JSON file
    let test_json = "/tmp/data_fetcher_test.json";
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    fs::write(test_json, serde_json::to_string_pretty(&users)?).await?;

    let fetcher = DataFetcher::new();

    // Read as dynamic JSON value
    let json_value = fetcher.read_json_value(test_json).await?;
    println!(
        "   JSON value: {}",
        serde_json::to_string_pretty(&json_value)?
    );

    // Read as typed struct
    let typed_users: Vec<User> = fetcher.read_json_file(test_json).await?;
    println!("   Typed users:");
    for user in typed_users {
        println!("     - {} ({}): {}", user.id, user.name, user.email);
    }

    // Cleanup
    std::fs::remove_file(test_json)?;
    println!();

    Ok(())
}

async fn demo_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. Error Handling");

    let fetcher = DataFetcher::new();

    // Test invalid URL
    match fetcher.get("not-a-valid-url").await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   Expected error for invalid URL: {}", e),
    }

    // Test nonexistent file
    match fetcher.read_text_file("/nonexistent/file.txt").await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   Expected error for missing file: {}", e),
    }

    // Test invalid JSON
    let invalid_json = "/tmp/invalid.json";
    fs::write(invalid_json, "{ this is not valid json }").await?;
    match fetcher.read_json_value(invalid_json).await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   Expected error for invalid JSON: {}", e),
    }
    std::fs::remove_file(invalid_json)?;

    println!();
    Ok(())
}

async fn demo_quick_functions() -> Result<(), Box<dyn std::error::Error>> {
    println!("7. Quick Convenience Functions");

    use periplon_sdk::data_fetcher::quick;

    // Quick GET
    println!("   Using quick::get()...");
    let response = quick::get("https://api.example.com/quick").await?;
    println!("   Response status: {}", response.status);

    // Quick file read
    let test_file = "/tmp/quick_test.txt";
    std::fs::write(test_file, "Quick read test")?;

    println!("   Using quick::read_file()...");
    let content = quick::read_file(test_file).await?;
    println!("   Content: {}", content.trim());

    std::fs::remove_file(test_file)?;

    println!();
    Ok(())
}
