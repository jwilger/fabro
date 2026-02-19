use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- 3.2 Role ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
    Developer,
}

// --- 3.4 ContentKind ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Text,
    Image,
    Audio,
    Document,
    ToolCall,
    ToolResult,
    Thinking,
    RedactedThinking,
    /// Extension for provider-specific content kinds
    Custom(String),
}

// --- 3.5 Content Data Structures ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageData {
    pub url: Option<String>,
    pub data: Option<Vec<u8>>,
    pub media_type: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioData {
    pub url: Option<String>,
    pub data: Option<Vec<u8>>,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentData {
    pub url: Option<String>,
    pub data: Option<Vec<u8>>,
    pub media_type: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallData {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    #[serde(default = "default_tool_type")]
    pub r#type: String,
}

fn default_tool_type() -> String {
    "function".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResultData {
    pub tool_call_id: String,
    pub content: serde_json::Value,
    pub is_error: bool,
    pub image_data: Option<Vec<u8>>,
    pub image_media_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingData {
    pub text: String,
    pub signature: Option<String>,
    pub redacted: bool,
}

// --- 3.3 ContentPart ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPart {
    pub kind: ContentKind,
    pub text: Option<String>,
    pub image: Option<ImageData>,
    pub audio: Option<AudioData>,
    pub document: Option<DocumentData>,
    pub tool_call: Option<ToolCallData>,
    pub tool_result: Option<ToolResultData>,
    pub thinking: Option<ThinkingData>,
}

impl ContentPart {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            kind: ContentKind::Text,
            text: Some(text.into()),
            image: None,
            audio: None,
            document: None,
            tool_call: None,
            tool_result: None,
            thinking: None,
        }
    }

    #[must_use] 
    pub const fn image(image: ImageData) -> Self {
        Self {
            kind: ContentKind::Image,
            text: None,
            image: Some(image),
            audio: None,
            document: None,
            tool_call: None,
            tool_result: None,
            thinking: None,
        }
    }

    #[must_use] 
    pub const fn tool_call(data: ToolCallData) -> Self {
        Self {
            kind: ContentKind::ToolCall,
            text: None,
            image: None,
            audio: None,
            document: None,
            tool_call: Some(data),
            tool_result: None,
            thinking: None,
        }
    }

    #[must_use] 
    pub const fn tool_result(data: ToolResultData) -> Self {
        Self {
            kind: ContentKind::ToolResult,
            text: None,
            image: None,
            audio: None,
            document: None,
            tool_call: None,
            tool_result: Some(data),
            thinking: None,
        }
    }

    #[must_use] 
    pub const fn thinking(data: ThinkingData) -> Self {
        Self {
            kind: ContentKind::Thinking,
            text: None,
            image: None,
            audio: None,
            document: None,
            tool_call: None,
            tool_result: None,
            thinking: Some(data),
        }
    }
}

// --- 3.1 Message ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentPart>,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentPart::text(text)],
            name: None,
            tool_call_id: None,
        }
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::text(text)],
            name: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::text(text)],
            name: None,
            tool_call_id: None,
        }
    }

    pub fn tool_result(
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        let id = tool_call_id.into();
        Self {
            role: Role::Tool,
            content: vec![ContentPart::tool_result(ToolResultData {
                tool_call_id: id.clone(),
                content: serde_json::Value::String(content.into()),
                is_error,
                image_data: None,
                image_media_type: None,
            })],
            name: None,
            tool_call_id: Some(id),
        }
    }

    /// Concatenates text from all text content parts.
    #[must_use] 
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter(|part| part.kind == ContentKind::Text)
            .filter_map(|part| part.text.as_deref())
            .collect::<Vec<_>>()
            .join("")
    }
}

// --- 3.8 FinishReason ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinishReason {
    pub reason: String,
    pub raw: Option<String>,
}

impl FinishReason {
    #[must_use] 
    pub fn stop() -> Self {
        Self {
            reason: "stop".to_string(),
            raw: None,
        }
    }

    #[must_use] 
    pub fn length() -> Self {
        Self {
            reason: "length".to_string(),
            raw: None,
        }
    }

    #[must_use] 
    pub fn tool_calls() -> Self {
        Self {
            reason: "tool_calls".to_string(),
            raw: None,
        }
    }

    #[must_use] 
    pub fn content_filter() -> Self {
        Self {
            reason: "content_filter".to_string(),
            raw: None,
        }
    }

    #[must_use] 
    pub fn error() -> Self {
        Self {
            reason: "error".to_string(),
            raw: None,
        }
    }

    pub fn other(raw: impl Into<String>) -> Self {
        let r = raw.into();
        Self {
            reason: "other".to_string(),
            raw: Some(r),
        }
    }

    #[must_use]
    pub fn with_raw(mut self, raw: impl Into<String>) -> Self {
        self.raw = Some(raw.into());
        self
    }
}

// --- 3.9 Usage ---

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub reasoning_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub raw: Option<serde_json::Value>,
}

impl std::ops::Add for Usage {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        const fn add_optional(a: Option<i64>, b: Option<i64>) -> Option<i64> {
            match (a, b) {
                (None, None) => None,
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (Some(a), Some(b)) => Some(a + b),
            }
        }

        Self {
            input_tokens: self.input_tokens + rhs.input_tokens,
            output_tokens: self.output_tokens + rhs.output_tokens,
            total_tokens: self.total_tokens + rhs.total_tokens,
            reasoning_tokens: add_optional(self.reasoning_tokens, rhs.reasoning_tokens),
            cache_read_tokens: add_optional(self.cache_read_tokens, rhs.cache_read_tokens),
            cache_write_tokens: add_optional(self.cache_write_tokens, rhs.cache_write_tokens),
            raw: None,
        }
    }
}

// --- 3.10 ResponseFormat ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseFormat {
    pub r#type: String,
    pub json_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub strict: bool,
}

// --- 3.11 Warning ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    pub message: String,
    pub code: Option<String>,
}

// --- 3.12 RateLimitInfo ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests_remaining: Option<i64>,
    pub requests_limit: Option<i64>,
    pub tokens_remaining: Option<i64>,
    pub tokens_limit: Option<i64>,
    pub reset_at: Option<String>,
}

// --- 3.6 Request ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub messages: Vec<Message>,
    pub provider: Option<String>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub response_format: Option<ResponseFormat>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub stop_sequences: Option<Vec<String>>,
    pub reasoning_effort: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub provider_options: Option<serde_json::Value>,
}

// --- 5.1 ToolDefinition ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// --- 5.3 ToolChoice ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolChoice {
    pub mode: String,
    pub tool_name: Option<String>,
}

impl ToolChoice {
    #[must_use] 
    pub fn auto() -> Self {
        Self {
            mode: "auto".to_string(),
            tool_name: None,
        }
    }

    #[must_use] 
    pub fn none() -> Self {
        Self {
            mode: "none".to_string(),
            tool_name: None,
        }
    }

    #[must_use] 
    pub fn required() -> Self {
        Self {
            mode: "required".to_string(),
            tool_name: None,
        }
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self {
            mode: "named".to_string(),
            tool_name: Some(name.into()),
        }
    }
}

// --- 5.4 ToolCall / ToolResult ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub raw_arguments: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: serde_json::Value,
    pub is_error: bool,
}

// --- 3.7 Response ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub model: String,
    pub provider: String,
    pub message: Message,
    pub finish_reason: FinishReason,
    pub usage: Usage,
    pub raw: Option<serde_json::Value>,
    pub warnings: Vec<Warning>,
    pub rate_limit: Option<RateLimitInfo>,
}

impl Response {
    #[must_use] 
    pub fn text(&self) -> String {
        self.message.text()
    }

    #[must_use] 
    pub fn tool_calls(&self) -> Vec<ToolCall> {
        self.message
            .content
            .iter()
            .filter(|part| part.kind == ContentKind::ToolCall)
            .filter_map(|part| {
                part.tool_call.as_ref().map(|tc| ToolCall {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    arguments: tc.arguments.clone(),
                    raw_arguments: None,
                })
            })
            .collect()
    }

    #[must_use] 
    pub fn reasoning(&self) -> Option<String> {
        let texts: Vec<&str> = self
            .message
            .content
            .iter()
            .filter(|part| part.kind == ContentKind::Thinking)
            .filter_map(|part| part.thinking.as_ref().map(|t| t.text.as_str()))
            .collect();

        if texts.is_empty() {
            None
        } else {
            Some(texts.join(""))
        }
    }
}

// --- 3.13 StreamEvent ---

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventType {
    StreamStart,
    TextStart,
    TextDelta,
    TextEnd,
    ReasoningStart,
    ReasoningDelta,
    ReasoningEnd,
    ToolCallStart,
    ToolCallDelta,
    ToolCallEnd,
    Finish,
    Error,
    ProviderEvent,
    /// Extension for custom event types
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub r#type: StreamEventType,
    pub delta: Option<String>,
    pub text_id: Option<String>,
    pub reasoning_delta: Option<String>,
    pub tool_call: Option<ToolCall>,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<Usage>,
    pub response: Option<Box<Response>>,
    pub error: Option<String>,
    pub raw: Option<serde_json::Value>,
}

impl StreamEvent {
    pub fn text_delta(delta: impl Into<String>, text_id: Option<String>) -> Self {
        Self {
            r#type: StreamEventType::TextDelta,
            delta: Some(delta.into()),
            text_id,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: None,
            usage: None,
            response: None,
            error: None,
            raw: None,
        }
    }

    #[must_use] 
    pub fn finish(reason: FinishReason, usage: Usage, response: Response) -> Self {
        Self {
            r#type: StreamEventType::Finish,
            delta: None,
            text_id: None,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: Some(reason),
            usage: Some(usage),
            response: Some(Box::new(response)),
            error: None,
            raw: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            r#type: StreamEventType::Error,
            delta: None,
            text_id: None,
            reasoning_delta: None,
            tool_call: None,
            finish_reason: None,
            usage: None,
            response: None,
            error: Some(message.into()),
            raw: None,
        }
    }
}

// --- 2.9 ModelInfo ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub context_window: i64,
    pub max_output: Option<i64>,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_reasoning: bool,
    pub input_cost_per_million: Option<f64>,
    pub output_cost_per_million: Option<f64>,
    pub aliases: Vec<String>,
}

// --- 4.7 Timeouts ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub total: Option<f64>,
    pub per_step: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterTimeout {
    pub connect: f64,
    pub request: f64,
    pub stream_read: f64,
}

impl Default for AdapterTimeout {
    fn default() -> Self {
        Self {
            connect: 10.0,
            request: 120.0,
            stream_read: 30.0,
        }
    }
}

// --- 6.6 RetryPolicy ---

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: f64,
    pub max_delay: f64,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay: 1.0,
            max_delay: 60.0,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    #[must_use] 
    pub fn delay_for_attempt(&self, attempt: u32) -> f64 {
        #[allow(clippy::cast_possible_wrap)]
        let delay = self.base_delay * self.backoff_multiplier.powi(attempt as i32);
        let delay = delay.min(self.max_delay);

        if self.jitter {
            let jitter_factor = 0.5 + rand::random::<f64>(); // 0.5..1.5
            delay * jitter_factor
        } else {
            delay
        }
    }
}

// --- 4.3 GenerateResult / StepResult ---

#[derive(Debug, Clone)]
pub struct GenerateResult {
    pub text: String,
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
    pub finish_reason: FinishReason,
    pub usage: Usage,
    pub total_usage: Usage,
    pub steps: Vec<StepResult>,
    pub response: Response,
    pub output: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub text: String,
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
    pub finish_reason: FinishReason,
    pub usage: Usage,
    pub response: Response,
    pub warnings: Vec<Warning>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_system_constructor() {
        let msg = Message::system("You are helpful.");
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.text(), "You are helpful.");
    }

    #[test]
    fn message_user_constructor() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text(), "Hello");
    }

    #[test]
    fn message_assistant_constructor() {
        let msg = Message::assistant("Hi there");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.text(), "Hi there");
    }

    #[test]
    fn message_tool_result_constructor() {
        let msg = Message::tool_result("call_123", "72F and sunny", false);
        assert_eq!(msg.role, Role::Tool);
        assert_eq!(msg.tool_call_id, Some("call_123".to_string()));
        let tr = msg.content[0].tool_result.as_ref().unwrap();
        assert_eq!(tr.tool_call_id, "call_123");
        assert!(!tr.is_error);
    }

    #[test]
    fn message_text_concatenates_text_parts() {
        let msg = Message {
            role: Role::Assistant,
            content: vec![
                ContentPart::text("Hello "),
                ContentPart::tool_call(ToolCallData {
                    id: "c1".into(),
                    name: "test".into(),
                    arguments: serde_json::json!({}),
                    r#type: "function".into(),
                }),
                ContentPart::text("world"),
            ],
            name: None,
            tool_call_id: None,
        };
        assert_eq!(msg.text(), "Hello world");
    }

    #[test]
    fn message_text_returns_empty_for_no_text_parts() {
        let msg = Message {
            role: Role::Assistant,
            content: vec![ContentPart::tool_call(ToolCallData {
                id: "c1".into(),
                name: "test".into(),
                arguments: serde_json::json!({}),
                r#type: "function".into(),
            })],
            name: None,
            tool_call_id: None,
        };
        assert_eq!(msg.text(), "");
    }

    #[test]
    fn finish_reason_constructors() {
        assert_eq!(FinishReason::stop().reason, "stop");
        assert_eq!(FinishReason::length().reason, "length");
        assert_eq!(FinishReason::tool_calls().reason, "tool_calls");
        assert_eq!(FinishReason::content_filter().reason, "content_filter");
        assert_eq!(FinishReason::error().reason, "error");
        let other = FinishReason::other("custom_reason");
        assert_eq!(other.reason, "other");
        assert_eq!(other.raw, Some("custom_reason".to_string()));
    }

    #[test]
    fn finish_reason_with_raw() {
        let fr = FinishReason::stop().with_raw("end_turn");
        assert_eq!(fr.reason, "stop");
        assert_eq!(fr.raw, Some("end_turn".to_string()));
    }

    #[test]
    fn usage_addition_both_filled() {
        let a = Usage {
            input_tokens: 10,
            output_tokens: 20,
            total_tokens: 30,
            reasoning_tokens: Some(5),
            cache_read_tokens: Some(3),
            cache_write_tokens: Some(1),
            raw: None,
        };
        let b = Usage {
            input_tokens: 15,
            output_tokens: 25,
            total_tokens: 40,
            reasoning_tokens: Some(10),
            cache_read_tokens: Some(7),
            cache_write_tokens: Some(2),
            raw: None,
        };
        let sum = a + b;
        assert_eq!(sum.input_tokens, 25);
        assert_eq!(sum.output_tokens, 45);
        assert_eq!(sum.total_tokens, 70);
        assert_eq!(sum.reasoning_tokens, Some(15));
        assert_eq!(sum.cache_read_tokens, Some(10));
        assert_eq!(sum.cache_write_tokens, Some(3));
    }

    #[test]
    fn usage_addition_one_none() {
        let a = Usage {
            input_tokens: 10,
            output_tokens: 20,
            total_tokens: 30,
            reasoning_tokens: Some(5),
            cache_read_tokens: None,
            cache_write_tokens: None,
            raw: None,
        };
        let b = Usage {
            input_tokens: 15,
            output_tokens: 25,
            total_tokens: 40,
            reasoning_tokens: None,
            cache_read_tokens: Some(7),
            cache_write_tokens: None,
            raw: None,
        };
        let sum = a + b;
        assert_eq!(sum.reasoning_tokens, Some(5));
        assert_eq!(sum.cache_read_tokens, Some(7));
        assert_eq!(sum.cache_write_tokens, None);
    }

    #[test]
    fn tool_choice_constructors() {
        assert_eq!(ToolChoice::auto().mode, "auto");
        assert_eq!(ToolChoice::none().mode, "none");
        assert_eq!(ToolChoice::required().mode, "required");
        let named = ToolChoice::named("get_weather");
        assert_eq!(named.mode, "named");
        assert_eq!(named.tool_name, Some("get_weather".to_string()));
    }

    #[test]
    fn response_text_accessor() {
        let response = Response {
            id: "resp_1".into(),
            model: "test-model".into(),
            provider: "test".into(),
            message: Message::assistant("Hello world"),
            finish_reason: FinishReason::stop(),
            usage: Usage::default(),
            raw: None,
            warnings: vec![],
            rate_limit: None,
        };
        assert_eq!(response.text(), "Hello world");
    }

    #[test]
    fn response_tool_calls_accessor() {
        let response = Response {
            id: "resp_1".into(),
            model: "test-model".into(),
            provider: "test".into(),
            message: Message {
                role: Role::Assistant,
                content: vec![
                    ContentPart::text("Let me check"),
                    ContentPart::tool_call(ToolCallData {
                        id: "call_1".into(),
                        name: "get_weather".into(),
                        arguments: serde_json::json!({"city": "SF"}),
                        r#type: "function".into(),
                    }),
                ],
                name: None,
                tool_call_id: None,
            },
            finish_reason: FinishReason::tool_calls(),
            usage: Usage::default(),
            raw: None,
            warnings: vec![],
            rate_limit: None,
        };
        let calls = response.tool_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "get_weather");
        assert_eq!(calls[0].id, "call_1");
    }

    #[test]
    fn response_reasoning_accessor() {
        let response = Response {
            id: "resp_1".into(),
            model: "test-model".into(),
            provider: "test".into(),
            message: Message {
                role: Role::Assistant,
                content: vec![
                    ContentPart::thinking(ThinkingData {
                        text: "Let me think...".into(),
                        signature: Some("sig_123".into()),
                        redacted: false,
                    }),
                    ContentPart::text("The answer is 42."),
                ],
                name: None,
                tool_call_id: None,
            },
            finish_reason: FinishReason::stop(),
            usage: Usage::default(),
            raw: None,
            warnings: vec![],
            rate_limit: None,
        };
        assert_eq!(response.reasoning(), Some("Let me think...".to_string()));
        assert_eq!(response.text(), "The answer is 42.");
    }

    #[test]
    fn response_reasoning_returns_none_when_absent() {
        let response = Response {
            id: "resp_1".into(),
            model: "test-model".into(),
            provider: "test".into(),
            message: Message::assistant("Hello"),
            finish_reason: FinishReason::stop(),
            usage: Usage::default(),
            raw: None,
            warnings: vec![],
            rate_limit: None,
        };
        assert_eq!(response.reasoning(), None);
    }

    #[test]
    fn stream_event_text_delta() {
        let event = StreamEvent::text_delta("hello", Some("t1".into()));
        assert_eq!(event.r#type, StreamEventType::TextDelta);
        assert_eq!(event.delta, Some("hello".to_string()));
        assert_eq!(event.text_id, Some("t1".to_string()));
    }

    #[test]
    fn stream_event_error() {
        let event = StreamEvent::error("something went wrong");
        assert_eq!(event.r#type, StreamEventType::Error);
        assert_eq!(event.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn retry_policy_delay_no_jitter() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: 1.0,
            max_delay: 60.0,
            backoff_multiplier: 2.0,
            jitter: false,
        };
        assert!((policy.delay_for_attempt(0) - 1.0).abs() < f64::EPSILON);
        assert!((policy.delay_for_attempt(1) - 2.0).abs() < f64::EPSILON);
        assert!((policy.delay_for_attempt(2) - 4.0).abs() < f64::EPSILON);
        assert!((policy.delay_for_attempt(3) - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn retry_policy_delay_respects_max() {
        let policy = RetryPolicy {
            max_retries: 10,
            base_delay: 1.0,
            max_delay: 5.0,
            backoff_multiplier: 2.0,
            jitter: false,
        };
        assert!((policy.delay_for_attempt(5) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn retry_policy_delay_with_jitter_in_range() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: 1.0,
            max_delay: 60.0,
            backoff_multiplier: 2.0,
            jitter: true,
        };
        let delay = policy.delay_for_attempt(0);
        // base * 0.5 to base * 1.5 => 0.5 to 1.5
        assert!(delay >= 0.5);
        assert!(delay <= 1.5);
    }

    #[test]
    fn adapter_timeout_defaults() {
        let timeout = AdapterTimeout::default();
        assert!((timeout.connect - 10.0).abs() < f64::EPSILON);
        assert!((timeout.request - 120.0).abs() < f64::EPSILON);
        assert!((timeout.stream_read - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn content_kind_custom() {
        let kind = ContentKind::Custom("provider_specific".into());
        assert_eq!(kind, ContentKind::Custom("provider_specific".into()));
    }

    #[test]
    fn content_part_text_constructor() {
        let part = ContentPart::text("hello");
        assert_eq!(part.kind, ContentKind::Text);
        assert_eq!(part.text, Some("hello".to_string()));
        assert!(part.image.is_none());
    }

    #[test]
    fn content_part_image_constructor() {
        let part = ContentPart::image(ImageData {
            url: Some("https://example.com/img.png".into()),
            data: None,
            media_type: None,
            detail: None,
        });
        assert_eq!(part.kind, ContentKind::Image);
        assert!(part.image.is_some());
    }

    #[test]
    fn tool_call_data_default_type() {
        let data: ToolCallData = serde_json::from_str(
            r#"{"id":"c1","name":"test","arguments":{}}"#,
        )
        .unwrap();
        assert_eq!(data.r#type, "function");
    }
}
