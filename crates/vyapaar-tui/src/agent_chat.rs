//! Agent chat transcript and markdown rendering.

use serde::{Deserialize, Serialize};
use vyapaar_highlight::{Language, highlight_code};

/// Who produced a chat message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// The user.
    User,
    /// The agent.
    Agent,
    /// The system (status, errors).
    System,
}

/// A single chat message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentMessage {
    /// The role that produced the message.
    pub role: AgentRole,
    /// The message text (markdown).
    pub text: String,
}

impl AgentMessage {
    /// Create a new message.
    pub fn new(role: AgentRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
        }
    }

    /// Highlight this message's markdown, returning styled spans.
    pub fn highlight(&self) -> Vec<vyapaar_highlight::HighlightSpan> {
        highlight_code(Language::Markdown, &self.text)
    }
}

/// The chat transcript (ordered messages).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatTranscript {
    /// Messages in display order (oldest first).
    pub messages: Vec<AgentMessage>,
}

impl ChatTranscript {
    /// Create an empty transcript.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Append a message.
    pub fn push(&mut self, message: AgentMessage) {
        self.messages.push(message);
    }

    /// The number of messages.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether the transcript is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// The last message, if any.
    pub fn last(&self) -> Option<&AgentMessage> {
        self.messages.last()
    }
}

impl Default for ChatTranscript {
    fn default() -> Self {
        Self::new()
    }
}
