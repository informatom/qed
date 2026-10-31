// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    // `qed mcp /path/to/site`: an AI assistant started the app to work on
    // a site (#105). Answer it over stdin and stdout, without a window.
    let mut args = std::env::args_os().skip(1);
    if args.next().is_some_and(|first| first == "mcp") {
        return qed_lib::serve_mcp(args.next());
    }
    qed_lib::run();
    ExitCode::SUCCESS
}
