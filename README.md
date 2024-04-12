# thisel
Thisel is Solidty REPL TUI build on top of [chisel](https://github.com/foundry-rs/foundry/tree/master/crates/chisel).

Features:
  - autocomplete
  - Vim-like editor 

## Usage

### REPL Commands

```text
⚒️ Chisel help
=============
General
        !help | !h - Display all commands
        !quit | !q - Quit Chisel
        !exec <command> [args] | !e <command> [args] - Execute a shell command and print the output

Session
        !clear | !c - Clear current session source
        !source | !so - Display the source code of the current session
        !save [id] | !s [id] - Save the current session to cache
        !load <id> | !l <id> - Load a previous session ID from cache
        !list | !ls - List all cached sessions
        !clearcache | !cc - Clear the chisel cache of all stored sessions
        !export | !ex - Export the current session source to a script file
        !fetch <addr> <name> | !fe <addr> <name> - Fetch the interface of a verified contract on Etherscan
        !edit - Open the current session in an editor

Environment
        !fork <url> | !f <url> - Fork an RPC for the current session. Supply 0 arguments to return to a local network
        !traces | !t - Enable / disable traces for the current session
        !calldata [data] | !cd [data] - Set calldata (`msg.data`) for the current session (appended after function selector). Clears it if no argument provided.

Debug
        !memdump | !md - Dump the raw memory of the current state
        !stackdump | !sd - Dump the raw stack of the current state
        !rawstack <var> | !rs <var> - Display the raw value of a variable's stack allocation. For variables that are > 32 bytes in length, this will display their memory pointer.
```

[`ratatui`]: https://crates.io/crates/ratatui
[`chisel`]: https://github.com/foundry-rs/foundry/tree/master/crates/chisel
[`tui-textarea`]: https://github.com/rhysd/tui-textarea
[`alloy`]: https://github.com/alloy-rs/alloy
