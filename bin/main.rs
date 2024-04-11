//! Chisel CLI
//!
//! This module contains the core readline loop for the Chisel CLI as well as the
//! executable's `main` function.

use chisel::{
    app::App,
    prelude::{ChiselCommand, ChiselDispatcher, DispatchResult, Editor, Mode, Transition},
    ui::ui,
};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use eyre::Context;
use foundry_cli::{
    handler,
    opts::CoreBuildArgs,
    utils::{self, LoadConfig},
};
use foundry_common::{evm::EvmArgs, fs};
use foundry_config::{
    figment::{
        value::{Dict, Map},
        Metadata, Profile, Provider,
    },
    Config,
};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::Terminal;
use std::{
    env,
    time::{Duration, Instant},
};

use std::path::PathBuf;
use tui_textarea::TextArea;
use yansi::Paint;

// Loads project's figment and merges the build cli arguments into it
foundry_config::merge_impl_figment_convert!(Chisel, opts, evm_opts);

const VERSION_MESSAGE: &str = concat!(env!("CARGO_PKG_VERSION"));

/// Fast, utilitarian, and verbose Solidity REPL.
#[derive(Debug, Parser)]
#[command(name = "chisel", version = VERSION_MESSAGE)]
pub struct Chisel {
    #[command(subcommand)]
    pub cmd: Option<ChiselSubcommand>,

    /// Path to a directory containing Solidity files to import, or path to a single Solidity file.
    ///
    /// These files will be evaluated before the top-level of the
    /// REPL, therefore functioning as a prelude
    #[arg(long, help_heading = "REPL options")]
    pub prelude: Option<PathBuf>,

    /// Disable the default `Vm` import.
    #[arg(long, help_heading = "REPL options", long_help = format!(
        "Disable the default `Vm` import.\n\n\
         The import is disabled by default if the Solc version is less than {}.",
        chisel::session_source::MIN_VM_VERSION
    ))]
    pub no_vm: bool,

    #[command(flatten)]
    pub opts: CoreBuildArgs,

    #[command(flatten)]
    pub evm_opts: EvmArgs,
}

/// Chisel binary subcommands
#[derive(Debug, Subcommand)]
pub enum ChiselSubcommand {
    /// List all cached sessions
    List,

    /// Load a cached session
    Load {
        /// The ID of the session to load.
        id: String,
    },

    /// View the source of a cached session
    View {
        /// The ID of the session to load.
        id: String,
    },

    /// Clear all cached chisel sessions from the cache directory
    ClearCache,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    handler::install();
    utils::subscriber();
    #[cfg(windows)]
    if !Paint::enable_windows_ascii() {
        Paint::disable()
    }

    utils::load_dotenv();

    // Parse command args
    let args = Chisel::parse();

    // Load configuration
    let (config, evm_opts) = args.load_config_and_evm_opts()?;

    // Create a new cli dispatcher
    let mut dispatcher = ChiselDispatcher::new(chisel::session_source::SessionSourceConfig {
        // Enable traces if any level of verbosity was passed
        traces: config.verbosity > 0,
        foundry_config: config,
        no_vm: args.no_vm,
        evm_opts,
        backend: None,
        calldata: None,
    })?;

    // Execute prelude Solidity source files
    evaluate_prelude(&mut dispatcher, args.prelude).await?;

    // Check for chisel subcommands
    match &args.cmd {
        Some(ChiselSubcommand::List) => {
            let sessions =
                Box::pin(dispatcher.dispatch_command(ChiselCommand::ListSessions, &[])).await;
            match sessions {
                DispatchResult::CommandSuccess(Some(session_list)) => {
                    println!("{session_list}");
                }
                DispatchResult::CommandFailed(e) => eprintln!("{e}"),
                _ => panic!("Unexpected result: Please report this bug."),
            }
            return Ok(());
        }
        Some(ChiselSubcommand::Load { id } | ChiselSubcommand::View { id }) => {
            // For both of these subcommands, we need to attempt to load the session from cache
            match Box::pin(dispatcher.dispatch_command(ChiselCommand::Load, &[id])).await {
                DispatchResult::CommandSuccess(_) => { /* Continue */ }
                DispatchResult::CommandFailed(e) => {
                    eprintln!("{e}");
                    return Ok(());
                }
                _ => panic!("Unexpected result! Please report this bug."),
            }

            // If the subcommand was `view`, print the source and exit.
            if matches!(args.cmd, Some(ChiselSubcommand::View { .. })) {
                match Box::pin(dispatcher.dispatch_command(ChiselCommand::Source, &[])).await {
                    DispatchResult::CommandSuccess(Some(source)) => {
                        println!("{source}");
                    }
                    _ => panic!("Unexpected result! Please report this bug."),
                }
                return Ok(());
            }
        }
        Some(ChiselSubcommand::ClearCache) => {
            match Box::pin(dispatcher.dispatch_command(ChiselCommand::ClearCache, &[])).await {
                DispatchResult::CommandSuccess(Some(msg)) => println!("{}", msg.green()),
                DispatchResult::CommandFailed(e) => eprintln!("{e}"),
                _ => panic!("Unexpected result! Please report this bug."),
            }
            return Ok(());
        }
        None => { /* No chisel subcommand present; Continue */ }
    }

    // // Create a new rustyline Editor
    // let mut rl = Editor::<SolidityHelper, _>::new()?;
    // rl.set_helper(Some(SolidityHelper::default()));

    // // automatically add lines to history
    // rl.set_auto_add_history(true);

    // // load history
    // if let Some(chisel_history) = chisel_history_file() {
    //     let _ = rl.load_history(&chisel_history);
    // }

    // Print welcome header
    println!(
        "Welcome to Chisel! Type `{}` to show available commands.",
        Paint::green("!help")
    );

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let tick_rate = Duration::from_millis(250);
    run_app(&mut terminal, app, &mut dispatcher, tick_rate).await?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    dispatcher: &mut ChiselDispatcher,
    tick_rate: Duration,
) -> std::io::Result<()> {
    let mut textarea = TextArea::default();
    textarea.set_block(Mode::Normal.block());
    textarea.set_cursor_style(Mode::Normal.cursor_style());

    let mut editor = Editor::new(Mode::Normal);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &textarea, &mut app))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            editor = match Box::pin(editor.transition(
                crossterm::event::read()?.into(),
                &mut textarea,
                dispatcher,
                &mut app,
            ))
            .await
            {
                Transition::Mode(mode) if editor.mode != mode => {
                    textarea.set_block(mode.block());
                    textarea.set_cursor_style(mode.cursor_style());
                    editor.change_mode(mode)
                }
                Transition::Nop | Transition::Mode(_) => editor,
                Transition::Pending(input) => editor.with_pending(input),
                Transition::Quit => break,
            };
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

/// [Provider] impl
impl Provider for Chisel {
    fn metadata(&self) -> Metadata {
        Metadata::named("Script Args Provider")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, foundry_config::figment::Error> {
        Ok(Map::from([(Config::selected_profile(), Dict::default())]))
    }
}

/// Evaluate a single Solidity line.
async fn dispatch_repl_line(dispatcher: &mut ChiselDispatcher, line: &str) -> bool {
    let r = Box::pin(dispatcher.dispatch(line)).await;
    r.is_error()
}

/// Evaluate multiple Solidity source files contained within a
/// Chisel prelude directory.
async fn evaluate_prelude(
    dispatcher: &mut ChiselDispatcher,
    maybe_prelude: Option<PathBuf>,
) -> eyre::Result<()> {
    let Some(prelude_dir) = maybe_prelude else {
        return Ok(());
    };
    if prelude_dir.is_file() {
        println!(
            "{} {}",
            Paint::yellow("Loading prelude source file:"),
            prelude_dir.display(),
        );
        Box::pin(load_prelude_file(dispatcher, prelude_dir)).await?;
        println!(
            "{}\n",
            Paint::green("Prelude source file loaded successfully!")
        );
    } else {
        let prelude_sources = fs::files_with_ext(prelude_dir, "sol");
        let print_success_msg = !prelude_sources.is_empty();
        for source_file in prelude_sources {
            println!(
                "{} {}",
                Paint::yellow("Loading prelude source file:"),
                source_file.display(),
            );
            Box::pin(load_prelude_file(dispatcher, source_file)).await?;
        }

        if print_success_msg {
            println!(
                "{}\n",
                Paint::green("All prelude source files loaded successfully!")
            );
        }
    }
    Ok(())
}

/// Loads a single Solidity file into the prelude.
async fn load_prelude_file(dispatcher: &mut ChiselDispatcher, file: PathBuf) -> eyre::Result<()> {
    let prelude = fs::read_to_string(file)
        .wrap_err("Could not load source file. Are you sure this path is correct?")?;
    Box::pin(dispatch_repl_line(dispatcher, &prelude)).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Chisel::command().debug_assert();
    }
}
