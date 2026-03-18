use std::sync::Arc;

use fabro_agent::tool_registry::RegisteredTool;
use fabro_interview::{AnswerValue, Interviewer, Question, QuestionType};
use fabro_llm::types::ToolDefinition;

/// Create an `ask_human` tool that lets an agent ask the user questions interactively.
///
/// The tool blocks on the `Interviewer::ask()` call and returns the user's response
/// as plain text. This enables multi-turn conversations within a single agent step.
pub fn make_ask_human_tool(interviewer: Arc<dyn Interviewer>, stage_id: String) -> RegisteredTool {
    RegisteredTool {
        definition: ToolDefinition {
            name: "ask_human".into(),
            description: "Ask the human user a question and wait for their response. Use this to conduct interactive conversations, interviews, or gather information from the user.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The question to ask the human user"
                    },
                    "context": {
                        "type": "string",
                        "description": "Optional context to display to the user before the question"
                    }
                },
                "required": ["question"]
            }),
        },
        executor: Arc::new(move |args, _ctx| {
            let interviewer = Arc::clone(&interviewer);
            let stage_id = stage_id.clone();
            Box::pin(async move {
                let question_text = args
                    .get("question")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing required parameter: question".to_string())?;

                let context = args.get("context").and_then(|v| v.as_str());

                let mut question = Question::new(question_text, QuestionType::Freeform);
                question.stage = stage_id;
                if let Some(ctx) = context {
                    question.context_display = Some(ctx.to_string());
                }

                let answer = interviewer.ask(question).await;

                match answer.value {
                    AnswerValue::Text(text) => Ok(text),
                    AnswerValue::Skipped => Err("User skipped the question".to_string()),
                    AnswerValue::Timeout => Err("Question timed out waiting for response".to_string()),
                    other => Ok(format!("{other:?}")),
                }
            })
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use fabro_agent::sandbox::{DirEntry, ExecResult, GrepOptions, Sandbox};
    use fabro_interview::{Answer, Question};

    /// Minimal no-op sandbox for tool context in tests.
    struct StubSandbox;

    #[async_trait::async_trait]
    impl Sandbox for StubSandbox {
        async fn read_file(
            &self,
            _path: &str,
            _offset: Option<usize>,
            _limit: Option<usize>,
        ) -> Result<String, String> {
            Ok(String::new())
        }
        async fn write_file(&self, _path: &str, _content: &str) -> Result<(), String> {
            Ok(())
        }
        async fn delete_file(&self, _path: &str) -> Result<(), String> {
            Ok(())
        }
        async fn file_exists(&self, _path: &str) -> Result<bool, String> {
            Ok(false)
        }
        async fn list_directory(
            &self,
            _path: &str,
            _depth: Option<usize>,
        ) -> Result<Vec<DirEntry>, String> {
            Ok(vec![])
        }
        async fn exec_command(
            &self,
            _command: &str,
            _timeout_ms: u64,
            _working_dir: Option<&str>,
            _env_vars: Option<&std::collections::HashMap<String, String>>,
            _cancel_token: Option<tokio_util::sync::CancellationToken>,
        ) -> Result<ExecResult, String> {
            Ok(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                timed_out: false,
                duration_ms: 0,
            })
        }
        async fn grep(
            &self,
            _pattern: &str,
            _path: &str,
            _options: &GrepOptions,
        ) -> Result<Vec<String>, String> {
            Ok(vec![])
        }
        async fn glob(&self, _pattern: &str, _path: Option<&str>) -> Result<Vec<String>, String> {
            Ok(vec![])
        }
        async fn download_file_to_local(&self, _remote: &str, _local: &Path) -> Result<(), String> {
            Ok(())
        }
        async fn upload_file_from_local(&self, _local: &Path, _remote: &str) -> Result<(), String> {
            Ok(())
        }
        async fn initialize(&self) -> Result<(), String> {
            Ok(())
        }
        async fn cleanup(&self) -> Result<(), String> {
            Ok(())
        }
        fn working_directory(&self) -> &str {
            "/workspace"
        }
        fn platform(&self) -> &str {
            "linux"
        }
        fn os_version(&self) -> String {
            "test".to_string()
        }
        async fn set_autostop_interval(&self, _minutes: i32) -> Result<(), String> {
            Ok(())
        }
    }

    fn make_ctx() -> fabro_agent::tool_registry::ToolContext {
        fabro_agent::tool_registry::ToolContext {
            env: Arc::new(StubSandbox),
            cancel: tokio_util::sync::CancellationToken::new(),
            tool_env: None,
        }
    }

    struct FakeInterviewer {
        response: String,
    }

    #[async_trait::async_trait]
    impl Interviewer for FakeInterviewer {
        async fn ask(&self, _question: Question) -> Answer {
            Answer {
                value: AnswerValue::Text(self.response.clone()),
                selected_option: None,
                selected_options: Vec::new(),
                text: Some(self.response.clone()),
            }
        }
    }

    struct SkippingInterviewer;

    #[async_trait::async_trait]
    impl Interviewer for SkippingInterviewer {
        async fn ask(&self, _question: Question) -> Answer {
            Answer {
                value: AnswerValue::Skipped,
                selected_option: None,
                selected_options: Vec::new(),
                text: None,
            }
        }
    }

    struct TimeoutInterviewer;

    #[async_trait::async_trait]
    impl Interviewer for TimeoutInterviewer {
        async fn ask(&self, _question: Question) -> Answer {
            Answer {
                value: AnswerValue::Timeout,
                selected_option: None,
                selected_options: Vec::new(),
                text: None,
            }
        }
    }

    #[test]
    fn tool_definition_correct() {
        let interviewer: Arc<dyn Interviewer> = Arc::new(FakeInterviewer {
            response: "test".to_string(),
        });
        let tool = make_ask_human_tool(interviewer, "stage1".to_string());

        assert_eq!(tool.definition.name, "ask_human");
        assert!(tool.definition.parameters["properties"]["question"].is_object());
        assert!(tool.definition.parameters["properties"]["context"].is_object());

        let required = tool.definition.parameters["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("question")));
        assert!(!required.contains(&serde_json::json!("context")));
    }

    #[tokio::test]
    async fn returns_text_response() {
        let interviewer: Arc<dyn Interviewer> = Arc::new(FakeInterviewer {
            response: "I need a web app".to_string(),
        });
        let tool = make_ask_human_tool(interviewer, "discovery".to_string());

        let result = (tool.executor)(
            serde_json::json!({"question": "What are you building?"}),
            make_ctx(),
        )
        .await;

        assert_eq!(result.unwrap(), "I need a web app");
    }

    #[tokio::test]
    async fn returns_error_on_skip() {
        let interviewer: Arc<dyn Interviewer> = Arc::new(SkippingInterviewer);
        let tool = make_ask_human_tool(interviewer, "stage".to_string());

        let result = (tool.executor)(serde_json::json!({"question": "Hello?"}), make_ctx()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("skipped"));
    }

    #[tokio::test]
    async fn returns_error_on_timeout() {
        let interviewer: Arc<dyn Interviewer> = Arc::new(TimeoutInterviewer);
        let tool = make_ask_human_tool(interviewer, "stage".to_string());

        let result = (tool.executor)(serde_json::json!({"question": "Hello?"}), make_ctx()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[tokio::test]
    async fn missing_question_returns_error() {
        let interviewer: Arc<dyn Interviewer> = Arc::new(FakeInterviewer {
            response: "test".to_string(),
        });
        let tool = make_ask_human_tool(interviewer, "stage".to_string());

        let result = (tool.executor)(serde_json::json!({}), make_ctx()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("question"));
    }
}
