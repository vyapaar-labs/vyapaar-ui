//! Chat transcript and message models for the Ratatui agent tab.

use serde::{Deserialize, Serialize};
use vyapaar_highlight::{HighlightSpan, Language, highlight_code};

/// The role of the speaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// The user asking questions.
    User,
    /// The agent answering.
    Agent,
    /// System notices.
    System,
}

/// A single message in the chat history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Who sent this message.
    pub role: AgentRole,
    /// The text content of the message.
    pub text: String,
}

impl AgentMessage {
    /// Construct a new message.
    pub fn new(role: AgentRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
        }
    }

    /// Highlight this message with Markdown semantics.
    pub fn highlight(&self) -> Vec<HighlightSpan> {
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

    /// Clear all messages from transcript.
    pub fn clear(&mut self) {
        self.messages.clear();
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

    /// The last message as mutable, if any.
    pub fn last_mut(&mut self) -> Option<&mut AgentMessage> {
        self.messages.last_mut()
    }
}

impl Default for ChatTranscript {
    fn default() -> Self {
        Self::new()
    }
}
