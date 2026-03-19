use std::io::Write;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;

use fabro_interview::{Answer, ConsoleInterviewer, Interviewer, Question, QuestionType};

use super::run_progress::ProgressUI;

/// An interviewer that provides a crossterm-based multi-line input experience
/// for interactive workflow node `ask_human` calls, while delegating other
/// question types to the inner `ConsoleInterviewer`.
///
/// Progress bars are hidden during prompts (same as `ProgressAwareInterviewer`).
pub struct ConversationInterviewer {
    inner: ConsoleInterviewer,
    progress: Arc<Mutex<ProgressUI>>,
}

impl ConversationInterviewer {
    pub fn new(inner: ConsoleInterviewer, progress: Arc<Mutex<ProgressUI>>) -> Self {
        Self { inner, progress }
    }
}

/// Read multi-line input from the terminal using crossterm raw mode.
///
/// - Printable chars are inserted and echoed
/// - Enter inserts a newline
/// - Backspace deletes the last char
/// - Ctrl+D submits the input
/// - Ctrl+C returns None (skipped)
fn read_multiline_input() -> std::io::Result<Option<String>> {
    let mut buffer = String::new();
    let mut stderr = std::io::stderr();

    terminal::enable_raw_mode()?;
    let _guard = scopeguard::guard((), |_| {
        let _ = terminal::disable_raw_mode();
    });

    loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match (code, modifiers) {
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    write!(stderr, "\r\n")?;
                    stderr.flush()?;
                    break;
                }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    write!(stderr, "\r\n")?;
                    stderr.flush()?;
                    return Ok(None);
                }
                (KeyCode::Enter, _) => {
                    buffer.push('\n');
                    write!(stderr, "\r\n    ")?;
                    stderr.flush()?;
                }
                (KeyCode::Backspace, _) => {
                    if let Some(ch) = buffer.pop() {
                        if ch == '\n' {
                            write!(stderr, "\x1b[A\x1b[999C")?;
                        } else {
                            write!(stderr, "\x08 \x08")?;
                        }
                        stderr.flush()?;
                    }
                }
                (KeyCode::Char(c), _) => {
                    buffer.push(c);
                    write!(stderr, "{c}")?;
                    stderr.flush()?;
                }
                _ => {}
            }
        }
    }

    Ok(Some(buffer))
}

#[async_trait]
impl Interviewer for ConversationInterviewer {
    async fn ask(&self, question: Question) -> Answer {
        self.hide_progress();

        // Render context if present
        if let Some(ref ctx) = question.context_display {
            eprint!("{ctx}");
        }

        // Print the question
        eprintln!("\x1b[1m{}\x1b[0m", question.text);

        // For non-freeform questions, delegate to inner interviewer
        if question.question_type != QuestionType::Freeform {
            let answer = self.inner.ask(question).await;
            self.show_progress();
            return answer;
        }

        // Multi-line input prompt
        eprint!("\x1b[36mYou:\x1b[0m ");
        let _ = std::io::stderr().flush();

        let result = tokio::task::spawn_blocking(read_multiline_input)
            .await
            .unwrap_or(Ok(None));

        let answer = match result {
            Ok(Some(text)) => Answer::text(text.trim()),
            Ok(None) => Answer::skipped(),
            Err(_) => {
                // Fallback on crossterm error: delegate to inner
                let answer = self.inner.ask(question).await;
                self.show_progress();
                return answer;
            }
        };

        self.show_progress();
        answer
    }

    async fn inform(&self, message: &str, stage: &str) {
        self.hide_progress();
        self.inner.inform(message, stage).await;
        self.show_progress();
    }

    fn hide_progress(&self) {
        let ui = self.progress.lock().expect("progress lock poisoned");
        ui.hide_draw_target();
    }

    fn show_progress(&self) {
        let ui = self.progress.lock().expect("progress lock poisoned");
        ui.show_draw_target();
    }
}
