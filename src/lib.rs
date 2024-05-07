/// REPL input dispatcher module
pub mod dispatcher;

/// Builtin Chisel commands
pub mod cmd;

// custom vim-like editor
pub mod editor;

pub mod solidity_helper;

pub mod history;

/// Chisel Environment Module
pub mod session;

/// Chisel Session Source wrapper
pub mod session_source;

/// REPL contract runner
pub mod runner;

/// REPL contract executor
pub mod executor;

/// Terminal ui widgets
pub mod ui;

/// Global App implementation
pub mod app;

/// Auto completion of
pub mod auto_complete;

/// regex macro
pub mod regex;

/// Prelude of all chisel modules
pub mod prelude {
    pub use crate::{
        app::*, auto_complete::*, cmd::*, dispatcher::*, editor::*, runner::*, session::*,
        session_source::*, solidity_helper::*, ui::*,
    };
}
