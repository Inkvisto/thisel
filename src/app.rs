use crate::prelude::{autocomplete, ChiselDispatcher, DispatchResult};
use ratatui::prelude::Color;
use ratatui::widgets::ScrollbarState;
use tui_textarea::TextArea;

/// App holds the state of the application
#[derive(Clone)]
pub struct App {
    /// Current value of the input box
    pub input: String,
    /// current user suggestion.
    pub current_suggestion: usize,
    /// History of recorded messages
    pub messages: Vec<String>,
    /// History of recorded messages
    pub suggestions: Vec<String>,
    /// Color of message
    pub fg_color: Color,
    // state of Output scroll state
    pub vertical_scroll_state: ScrollbarState,
    // position of Output scroll
    pub vertical_scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> App {
        App {
            input: String::new(),
            messages: Vec::new(),
            suggestions: Vec::new(),
            current_suggestion: 0,
            fg_color: Color::Gray,
            vertical_scroll: 0,
            vertical_scroll_state: ScrollbarState::default(),
        }
    }

    pub fn suggest(&mut self, input: &str) {
        let suggestion = autocomplete(input);
        self.suggestions = suggestion;
    }

    fn clear_suggestions(&mut self){
        
        let ansi_escape_regex = crate::regex!(r"\x1B\[[0-?]*[ -/]*[@-~]");
        for suggestion in &mut self.suggestions {
            let result = ansi_escape_regex.replace_all(suggestion, "");
            *suggestion = result.to_string();
        }
    }

    pub fn choose_suggestion(&mut self, textarea: &mut TextArea) {
        let ansi_escape_regex = crate::regex!(r"\x1B\[[0-?]*[ -/]*[@-~]");
        self.clear_suggestions();
        if let Some(suggestion) = self.suggestions.get(self.current_suggestion) {
            let cleared_suggestion = ansi_escape_regex.replace_all(suggestion, "").to_string();

            textarea.delete_word();
            textarea.insert_str(&cleared_suggestion);
        }
    }

    pub fn iter_suggestion(&mut self) {
        self.current_suggestion = (self.current_suggestion + 1) % self.suggestions.len();

        self.highlight_current_suggestion();
    }

    pub fn highlight_current_suggestion(&mut self) {
        use yansi::Paint;
        self.clear_suggestions();
        let current_index = self.current_suggestion;
        self.suggestions[current_index] = self.suggestions[current_index].on_red().to_string();
    }

    pub async fn submit_message(&mut self, lines: &[String], dispatcher: &mut ChiselDispatcher) {
        let r = Box::pin(dispatcher.dispatch(&lines.join(""))).await;

        self.messages.clear();
        match &r {
            DispatchResult::Success(msg) | DispatchResult::CommandSuccess(msg) => {
                if let Some(msg) = msg {
                    self.messages.push(msg.to_string());
                    self.fg_color = Color::Green;
                }
            }
            DispatchResult::UnrecognizedCommand(e) => self.messages.push(e.to_string()),
            DispatchResult::SolangParserFailed(parser_error) => {
                for diagnostic in parser_error {
                    self.messages.push(diagnostic.message.clone());
                }
                self.fg_color = Color::Red;
            }
            DispatchResult::FileIoError(e) => {
                self.messages.push(e.to_string());
                self.fg_color = Color::Red;
            }
            DispatchResult::CommandFailed(msg) | DispatchResult::Failure(Some(msg)) => {
                self.messages.push(msg.to_string());
                self.fg_color = Color::Red;
            }
            DispatchResult::Failure(None) => self.messages.push(
                "⚒️ Unknown Chisel Error ⚒️\nPlease Report this bug as a git:w
             hub issue if it persists: https://github.com/foundry-rs/foundry/issues/new/choose"
                    .to_string(),
            ),
        }
    }
}
