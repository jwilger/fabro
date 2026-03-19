use tokio::sync::oneshot;

use fabro_interview::Answer;

use super::run_state::ToolCallState;

/// A message in the interactive chat view.
#[derive(Debug, Clone)]
pub enum ChatMessage {
    System {
        text: String,
    },
    Assistant {
        text: String,
        tool_calls: Vec<ToolCallState>,
        streaming: bool,
    },
    User {
        text: String,
    },
}

/// A pending question from TuiInterviewer, waiting for the user's answer.
pub struct PendingQuestion {
    pub stage_id: String,
    pub question_text: String,
    pub answer_tx: oneshot::Sender<Answer>,
}

/// Simple hand-rolled multi-line input buffer (replaces tui-textarea).
#[derive(Debug, Clone)]
pub struct InputBuffer {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
        }
    }
}

impl InputBuffer {
    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_row];
        let byte_idx = char_to_byte_idx(line, self.cursor_col);
        line.insert(byte_idx, c);
        self.cursor_col += 1;
    }

    /// Insert a newline at the cursor position.
    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.cursor_row];
        let byte_idx = char_to_byte_idx(line, self.cursor_col);
        let rest = line[byte_idx..].to_string();
        line.truncate(byte_idx);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.lines.insert(self.cursor_row, rest);
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let byte_idx = char_to_byte_idx(line, self.cursor_col);
            let prev_byte_idx = char_to_byte_idx(line, self.cursor_col - 1);
            line.drain(prev_byte_idx..byte_idx);
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].chars().count();
            self.lines[self.cursor_row].push_str(&current_line);
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete(&mut self) {
        let line = &self.lines[self.cursor_row];
        let char_count = line.chars().count();
        if self.cursor_col < char_count {
            let line = &mut self.lines[self.cursor_row];
            let byte_idx = char_to_byte_idx(line, self.cursor_col);
            let next_byte_idx = char_to_byte_idx(line, self.cursor_col + 1);
            line.drain(byte_idx..next_byte_idx);
        } else if self.cursor_row + 1 < self.lines.len() {
            // Merge next line into current
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].chars().count();
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        let char_count = self.lines[self.cursor_row].chars().count();
        if self.cursor_col < char_count {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let char_count = self.lines[self.cursor_row].chars().count();
            self.cursor_col = self.cursor_col.min(char_count);
        }
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            let char_count = self.lines[self.cursor_row].chars().count();
            self.cursor_col = self.cursor_col.min(char_count);
        }
    }

    /// Move cursor to start of line.
    pub fn home(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line.
    pub fn end(&mut self) {
        self.cursor_col = self.lines[self.cursor_row].chars().count();
    }

    /// Get the full text content.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Reset to empty.
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_row = 0;
        self.cursor_col = 0;
    }
}

/// Convert a char index to a byte index within a string.
fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// State for the interactive chat view.
pub struct ChatState {
    pub stage_id: String,
    pub messages: Vec<ChatMessage>,
    pub input: InputBuffer,
    pub scroll_offset: usize,
    pub pending_answer_tx: Option<oneshot::Sender<Answer>>,
}

impl ChatState {
    pub fn new(stage_id: String) -> Self {
        Self {
            stage_id,
            messages: Vec::new(),
            input: InputBuffer::default(),
            scroll_offset: 0,
            pending_answer_tx: None,
        }
    }

    /// Submit the current input as a user message and send the answer.
    pub fn submit(&mut self) -> bool {
        let text = self.input.text().trim().to_string();
        if text.is_empty() {
            return false;
        }

        self.messages.push(ChatMessage::User { text: text.clone() });

        if let Some(tx) = self.pending_answer_tx.take() {
            let _ = tx.send(Answer::text(text));
        }

        self.input.clear();
        true
    }

    /// Skip/cancel the current question.
    pub fn skip(&mut self) {
        if let Some(tx) = self.pending_answer_tx.take() {
            let _ = tx.send(Answer::skipped());
        }
    }
}

impl Drop for ChatState {
    fn drop(&mut self) {
        // If there's an unanswered question, skip it so the engine doesn't hang
        if let Some(tx) = self.pending_answer_tx.take() {
            let _ = tx.send(Answer::skipped());
        }
    }
}
