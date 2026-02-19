# unified-llm

A unified Rust client library for multiple LLM providers (OpenAI, Anthropic, Google Gemini). Write provider-agnostic code and switch models by changing a single string identifier.

## Architecture

The library is organized into four layers:

```
Layer 4: High-Level API         generate(), stream(), generate_object()
Layer 3: Core Client            Client, provider routing, middleware hooks
Layer 2: Provider Utilities     Shared helpers (SSE parsing, retry, etc.)
Layer 1: Provider Specification ProviderAdapter trait, shared types
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
unified-llm = { path = "crates/unified-llm" }
tokio = { version = "1", features = ["full"] }
```

## Usage Examples

### Simple Generation

```rust
use unified_llm::generate::{generate, GenerateParams};

#[tokio::main]
async fn main() {
    let result = generate(
        GenerateParams::new("claude-opus-4-6")
            .prompt("Explain quantum computing in one paragraph")
    ).await.unwrap();

    println!("{}", result.text);
    println!("Tokens used: {}", result.usage.total_tokens);
}
```

### Generation with System Message

```rust
use unified_llm::generate::{generate, GenerateParams};

let result = generate(
    GenerateParams::new("claude-opus-4-6")
        .system("You are a helpful coding assistant.")
        .prompt("Write a Rust function to check if a number is prime")
).await.unwrap();
```

### Generation with Tools

```rust
use unified_llm::generate::{generate, GenerateParams};
use unified_llm::tools::Tool;

let weather_tool = Tool::active(
    "get_weather",
    "Get the current weather for a location",
    serde_json::json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City name, e.g. 'San Francisco, CA'"
            }
        },
        "required": ["location"]
    }),
    |args| async move {
        let location = args["location"].as_str().unwrap_or("unknown");
        Ok(serde_json::json!(format!("72F and sunny in {}", location)))
    },
);

let result = generate(
    GenerateParams::new("claude-opus-4-6")
        .system("You are a helpful assistant with access to weather data.")
        .prompt("What is the weather in San Francisco?")
        .tools(vec![weather_tool])
        .max_tool_rounds(5)
).await.unwrap();

println!("{}", result.text);
println!("Steps taken: {}", result.steps.len());
println!("Total tokens: {}", result.total_usage.total_tokens);
```

### Streaming

```rust
use unified_llm::generate::{stream_generate, GenerateParams};
use unified_llm::types::StreamEventType;
use futures::StreamExt;

let mut stream = stream_generate(
    GenerateParams::new("claude-opus-4-6")
        .prompt("Write a haiku about coding")
).await.unwrap();

while let Some(event) = stream.next().await {
    let event = event.unwrap();
    if event.r#type == StreamEventType::TextDelta {
        print!("{}", event.delta.unwrap_or_default());
    }
}
```

### Structured Output

```rust
use unified_llm::generate::{generate_object, GenerateParams};

let schema = serde_json::json!({
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" }
    },
    "required": ["name", "age"]
});

let result = generate_object(
    GenerateParams::new("gpt-5.2")
        .prompt("Extract: 'Alice is 30 years old'"),
    schema,
).await.unwrap();

let output = result.output.unwrap();
assert_eq!(output["name"], "Alice");
assert_eq!(output["age"], 30);
```

### Client Configuration

```rust
use unified_llm::client::Client;
use std::sync::Arc;

// From environment variables (reads OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)
let client = Client::from_env();

// Or configure explicitly
let mut client = Client::new(
    std::collections::HashMap::new(),
    None,
    vec![],
);
// Register adapters...

// Use with generate
let result = generate(
    GenerateParams::new("claude-opus-4-6")
        .prompt("Hello")
        .client(Arc::new(client))
).await.unwrap();
```

### Model Catalog

```rust
use unified_llm::catalog::{get_model_info, list_models, get_latest_model};

// Look up a model
let info = get_model_info("claude-opus-4-6").unwrap();
println!("{} ({})", info.display_name, info.provider);
println!("Context window: {} tokens", info.context_window);

// Look up by alias
let info = get_model_info("opus").unwrap();
assert_eq!(info.id, "claude-opus-4-6");

// List all models for a provider
let anthropic_models = list_models(Some("anthropic"));
for model in &anthropic_models {
    println!("  {} - {}", model.id, model.display_name);
}

// Get the latest model for a provider
let best = get_latest_model("openai", Some("reasoning")).unwrap();
println!("Best OpenAI reasoning model: {}", best.id);
```

### Retry Logic

```rust
use unified_llm::retry::retry;
use unified_llm::types::RetryPolicy;

let policy = RetryPolicy {
    max_retries: 3,
    base_delay: 1.0,
    max_delay: 60.0,
    backoff_multiplier: 2.0,
    jitter: true,
};

let response = retry(&policy, || {
    let c = client.clone();
    let r = request.clone();
    async move { c.complete(&r).await }
}).await.unwrap();
```

### Error Handling

```rust
use unified_llm::error::SdkError;

match result {
    Ok(response) => println!("{}", response.text()),
    Err(SdkError::RateLimit { retry_after, .. }) => {
        println!("Rate limited. Retry after {:?}s", retry_after);
    }
    Err(SdkError::Authentication { message, .. }) => {
        println!("Auth error: {}", message);
    }
    Err(e) if e.retryable() => {
        println!("Transient error, can retry: {}", e);
    }
    Err(e) => {
        println!("Fatal error: {}", e);
    }
}
```

## Modules

| Module | Description |
|--------|-------------|
| `types` | Core data types: Message, Request, Response, Usage, StreamEvent, etc. |
| `error` | Error hierarchy with retryability classification |
| `client` | Client with provider routing and middleware |
| `provider` | ProviderAdapter trait |
| `middleware` | Middleware trait for cross-cutting concerns |
| `tools` | Tool definitions and parallel execution |
| `retry` | Retry with exponential backoff and jitter |
| `generate` | High-level API: generate(), stream(), generate_object() |
| `catalog` | Model catalog with lookup functions |

## Supported Providers

| Provider | API | Environment Variable |
|----------|-----|---------------------|
| OpenAI | Responses API (`/v1/responses`) | `OPENAI_API_KEY` |
| Anthropic | Messages API (`/v1/messages`) | `ANTHROPIC_API_KEY` |
| Gemini | Gemini API (`/v1beta/...`) | `GEMINI_API_KEY` |

## License

MIT
