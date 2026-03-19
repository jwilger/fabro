use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

use fabro_interview::{Answer, Interviewer, Question};

use crate::state::chat_state::PendingQuestion;

/// An `Interviewer` implementation that forwards questions to the TUI
/// via an mpsc channel and waits for the user's answer via a oneshot channel.
pub struct TuiInterviewer {
    chat_tx: mpsc::UnboundedSender<PendingQuestion>,
}

impl TuiInterviewer {
    pub fn new(chat_tx: mpsc::UnboundedSender<PendingQuestion>) -> Self {
        Self { chat_tx }
    }
}

#[async_trait]
impl Interviewer for TuiInterviewer {
    async fn ask(&self, question: Question) -> Answer {
        let (answer_tx, answer_rx) = oneshot::channel();
        let pending = PendingQuestion {
            stage_id: question.stage.clone(),
            question_text: question.text.clone(),
            answer_tx,
        };

        if self.chat_tx.send(pending).is_err() {
            return Answer::skipped();
        }

        match answer_rx.await {
            Ok(answer) => answer,
            Err(_) => Answer::skipped(),
        }
    }
}
