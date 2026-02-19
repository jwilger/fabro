use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    // --- Provider errors ---
    #[error("Authentication error from {provider}: {message}")]
    Authentication {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Access denied from {provider}: {message}")]
    AccessDenied {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Not found from {provider}: {message}")]
    NotFound {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Invalid request to {provider}: {message}")]
    InvalidRequest {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Rate limited by {provider}: {message}")]
    RateLimit {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Server error from {provider}: {message}")]
    Server {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Content filtered by {provider}: {message}")]
    ContentFilter {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Context length exceeded for {provider}: {message}")]
    ContextLength {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    #[error("Quota exceeded for {provider}: {message}")]
    QuotaExceeded {
        message: String,
        provider: String,
        status_code: Option<u16>,
        error_code: Option<String>,
        retry_after: Option<f64>,
        raw: Option<serde_json::Value>,
    },

    // --- Non-provider errors ---
    #[error("Request timed out: {message}")]
    RequestTimeout { message: String },

    #[error("Request aborted: {message}")]
    Abort { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Stream error: {message}")]
    Stream { message: String },

    #[error("Invalid tool call: {message}")]
    InvalidToolCall { message: String },

    #[error("No object generated: {message}")]
    NoObjectGenerated { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },
}

impl SdkError {
    #[must_use]
    pub const fn retryable(&self) -> bool {
        match self {
            Self::RateLimit { .. }
            | Self::Server { .. }
            | Self::RequestTimeout { .. }
            | Self::Network { .. }
            | Self::Stream { .. } => true,

            Self::Authentication { .. }
            | Self::AccessDenied { .. }
            | Self::NotFound { .. }
            | Self::InvalidRequest { .. }
            | Self::ContextLength { .. }
            | Self::QuotaExceeded { .. }
            | Self::ContentFilter { .. }
            | Self::Configuration { .. }
            | Self::Abort { .. }
            | Self::InvalidToolCall { .. }
            | Self::NoObjectGenerated { .. } => false,
        }
    }

    #[must_use] 
    pub const fn retry_after(&self) -> Option<f64> {
        match self {
            Self::RateLimit { retry_after, .. }
            | Self::Server { retry_after, .. }
            | Self::Authentication { retry_after, .. }
            | Self::AccessDenied { retry_after, .. }
            | Self::NotFound { retry_after, .. }
            | Self::InvalidRequest { retry_after, .. }
            | Self::ContentFilter { retry_after, .. }
            | Self::ContextLength { retry_after, .. }
            | Self::QuotaExceeded { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    #[must_use] 
    pub const fn status_code(&self) -> Option<u16> {
        match self {
            Self::Authentication { status_code, .. }
            | Self::AccessDenied { status_code, .. }
            | Self::NotFound { status_code, .. }
            | Self::InvalidRequest { status_code, .. }
            | Self::RateLimit { status_code, .. }
            | Self::Server { status_code, .. }
            | Self::ContentFilter { status_code, .. }
            | Self::ContextLength { status_code, .. }
            | Self::QuotaExceeded { status_code, .. } => *status_code,
            _ => None,
        }
    }
}

/// HTTP status code to error type mapping (Section 6.4).
#[must_use] 
pub fn error_from_status_code(
    status_code: u16,
    message: String,
    provider: String,
    error_code: Option<String>,
    raw: Option<serde_json::Value>,
    retry_after: Option<f64>,
) -> SdkError {
    // First check message-based classification for ambiguous cases
    let lower_msg = message.to_lowercase();
    if lower_msg.contains("context length") || lower_msg.contains("too many tokens") {
        return SdkError::ContextLength {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        };
    }
    if lower_msg.contains("content filter") || lower_msg.contains("safety") {
        return SdkError::ContentFilter {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        };
    }

    match status_code {
        400 | 422 => SdkError::InvalidRequest {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        },
        401 => SdkError::Authentication {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        },
        403 => SdkError::AccessDenied {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        },
        404 => SdkError::NotFound {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        },
        408 => SdkError::RequestTimeout { message },
        413 => SdkError::ContextLength {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        },
        429 => SdkError::RateLimit {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        },
        _ => SdkError::Server {
            message,
            provider,
            status_code: Some(status_code),
            error_code,
            retry_after,
            raw,
        },
    }
}

type GrpcErrorFactory =
    fn(String, String, Option<String>, Option<serde_json::Value>, Option<f64>) -> SdkError;

/// gRPC status code to error type mapping (Section 6.4, for Gemini).
static GRPC_STATUS_MAP: &[(&str, GrpcErrorFactory)] = &[
    ("NOT_FOUND", |msg, prov, code, raw, ra| SdkError::NotFound {
        message: msg, provider: prov, status_code: None, error_code: code, retry_after: ra, raw,
    }),
    ("INVALID_ARGUMENT", |msg, prov, code, raw, ra| SdkError::InvalidRequest {
        message: msg, provider: prov, status_code: None, error_code: code, retry_after: ra, raw,
    }),
    ("UNAUTHENTICATED", |msg, prov, code, raw, ra| SdkError::Authentication {
        message: msg, provider: prov, status_code: None, error_code: code, retry_after: ra, raw,
    }),
    ("PERMISSION_DENIED", |msg, prov, code, raw, ra| SdkError::AccessDenied {
        message: msg, provider: prov, status_code: None, error_code: code, retry_after: ra, raw,
    }),
    ("RESOURCE_EXHAUSTED", |msg, prov, code, raw, ra| SdkError::RateLimit {
        message: msg, provider: prov, status_code: None, error_code: code, retry_after: ra, raw,
    }),
    ("UNAVAILABLE", |msg, prov, code, raw, ra| SdkError::Server {
        message: msg, provider: prov, status_code: None, error_code: code, retry_after: ra, raw,
    }),
    ("DEADLINE_EXCEEDED", |msg, _prov, _code, _raw, _ra| SdkError::RequestTimeout {
        message: msg,
    }),
    ("INTERNAL", |msg, prov, code, raw, ra| SdkError::Server {
        message: msg, provider: prov, status_code: None, error_code: code, retry_after: ra, raw,
    }),
];

pub fn error_from_grpc_status(
    grpc_code: &str,
    message: String,
    provider: String,
    error_code: Option<String>,
    raw: Option<serde_json::Value>,
    retry_after: Option<f64>,
) -> SdkError {
    let grpc_map: HashMap<&str, &GrpcErrorFactory> =
        GRPC_STATUS_MAP.iter().map(|(k, f)| (*k, f)).collect();

    if let Some(factory) = grpc_map.get(grpc_code) {
        factory(message, provider, error_code, raw, retry_after)
    } else {
        SdkError::Server {
            message,
            provider,
            status_code: None,
            error_code,
            retry_after,
            raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_classification() {
        let auth_err = SdkError::Authentication {
            message: "bad key".into(),
            provider: "openai".into(),
            status_code: Some(401),
            error_code: None,
            retry_after: None,
            raw: None,
        };
        assert!(!auth_err.retryable());

        let rate_err = SdkError::RateLimit {
            message: "too fast".into(),
            provider: "openai".into(),
            status_code: Some(429),
            error_code: None,
            retry_after: Some(2.0),
            raw: None,
        };
        assert!(rate_err.retryable());
        assert_eq!(rate_err.retry_after(), Some(2.0));

        let server_err = SdkError::Server {
            message: "internal error".into(),
            provider: "anthropic".into(),
            status_code: Some(500),
            error_code: None,
            retry_after: None,
            raw: None,
        };
        assert!(server_err.retryable());

        let timeout = SdkError::RequestTimeout {
            message: "timed out".into(),
        };
        assert!(timeout.retryable());

        let network = SdkError::Network {
            message: "connection refused".into(),
        };
        assert!(network.retryable());

        let config = SdkError::Configuration {
            message: "missing provider".into(),
        };
        assert!(!config.retryable());
    }

    #[test]
    fn non_retryable_errors() {
        let errors: Vec<SdkError> = vec![
            SdkError::AccessDenied {
                message: "forbidden".into(),
                provider: "openai".into(),
                status_code: Some(403),
                error_code: None,
                retry_after: None,
                raw: None,
            },
            SdkError::NotFound {
                message: "not found".into(),
                provider: "openai".into(),
                status_code: Some(404),
                error_code: None,
                retry_after: None,
                raw: None,
            },
            SdkError::InvalidRequest {
                message: "bad request".into(),
                provider: "openai".into(),
                status_code: Some(400),
                error_code: None,
                retry_after: None,
                raw: None,
            },
            SdkError::ContextLength {
                message: "too long".into(),
                provider: "openai".into(),
                status_code: Some(413),
                error_code: None,
                retry_after: None,
                raw: None,
            },
            SdkError::QuotaExceeded {
                message: "quota".into(),
                provider: "openai".into(),
                status_code: None,
                error_code: None,
                retry_after: None,
                raw: None,
            },
            SdkError::ContentFilter {
                message: "blocked".into(),
                provider: "openai".into(),
                status_code: None,
                error_code: None,
                retry_after: None,
                raw: None,
            },
        ];
        for err in &errors {
            assert!(!err.retryable(), "Expected non-retryable: {err}");
        }
    }

    #[test]
    fn error_from_status_code_mapping() {
        let err = error_from_status_code(
            401,
            "unauthorized".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::Authentication { .. }));
        assert!(!err.retryable());

        let err = error_from_status_code(
            403,
            "forbidden".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::AccessDenied { .. }));

        let err = error_from_status_code(
            404,
            "not found".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::NotFound { .. }));

        let err = error_from_status_code(
            400,
            "bad request".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::InvalidRequest { .. }));

        let err = error_from_status_code(
            422,
            "unprocessable".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::InvalidRequest { .. }));

        let err = error_from_status_code(
            408,
            "timeout".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::RequestTimeout { .. }));

        let err = error_from_status_code(
            413,
            "too large".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::ContextLength { .. }));

        let err = error_from_status_code(
            429,
            "rate limited".into(),
            "openai".into(),
            None,
            None,
            Some(5.0),
        );
        assert!(matches!(err, SdkError::RateLimit { .. }));
        assert!(err.retryable());
        assert_eq!(err.retry_after(), Some(5.0));

        let err = error_from_status_code(
            500,
            "internal".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::Server { .. }));
        assert!(err.retryable());

        let err = error_from_status_code(
            502,
            "bad gateway".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::Server { .. }));
    }

    #[test]
    fn error_message_classification_context_length() {
        let err = error_from_status_code(
            400,
            "This model's maximum context length is 4096 tokens".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::ContextLength { .. }));
    }

    #[test]
    fn error_message_classification_too_many_tokens() {
        let err = error_from_status_code(
            400,
            "too many tokens in the request".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::ContextLength { .. }));
    }

    #[test]
    fn error_message_classification_content_filter() {
        let err = error_from_status_code(
            400,
            "Output blocked by content filter".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::ContentFilter { .. }));
    }

    #[test]
    fn error_message_classification_safety() {
        let err = error_from_status_code(
            400,
            "Response blocked due to safety concerns".into(),
            "openai".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::ContentFilter { .. }));
    }

    #[test]
    fn grpc_status_mapping() {
        let err = error_from_grpc_status(
            "NOT_FOUND",
            "model not found".into(),
            "gemini".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::NotFound { .. }));

        let err = error_from_grpc_status(
            "RESOURCE_EXHAUSTED",
            "rate limited".into(),
            "gemini".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::RateLimit { .. }));
        assert!(err.retryable());

        let err = error_from_grpc_status(
            "UNAUTHENTICATED",
            "bad key".into(),
            "gemini".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::Authentication { .. }));

        let err = error_from_grpc_status(
            "DEADLINE_EXCEEDED",
            "timeout".into(),
            "gemini".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::RequestTimeout { .. }));

        let err = error_from_grpc_status(
            "UNKNOWN_CODE",
            "something".into(),
            "gemini".into(),
            None,
            None,
            None,
        );
        assert!(matches!(err, SdkError::Server { .. }));
    }

    #[test]
    fn error_display_messages() {
        let err = SdkError::Authentication {
            message: "invalid api key".into(),
            provider: "openai".into(),
            status_code: Some(401),
            error_code: None,
            retry_after: None,
            raw: None,
        };
        assert_eq!(
            err.to_string(),
            "Authentication error from openai: invalid api key"
        );

        let err = SdkError::Configuration {
            message: "no provider".into(),
        };
        assert_eq!(err.to_string(), "Configuration error: no provider");
    }

    #[test]
    fn status_code_accessor() {
        let err = SdkError::Server {
            message: "error".into(),
            provider: "openai".into(),
            status_code: Some(503),
            error_code: None,
            retry_after: None,
            raw: None,
        };
        assert_eq!(err.status_code(), Some(503));

        let err = SdkError::Network {
            message: "refused".into(),
        };
        assert_eq!(err.status_code(), None);
    }
}
