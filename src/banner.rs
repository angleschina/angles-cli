/// Angles Code CLI — ASCII Art Banner with italic support
///
/// Prints a large "ANGLES CODE" banner at startup, using terminal escape
/// sequences for italic text when supported, falling back to bold/plain.

use std::env;
use std::io::{self, Write};

/// Check if the current terminal supports italic text.
fn supports_italic() -> bool {
    if cfg!(windows) && env::var("WT_SESSION").is_ok() {
        return true;
    }
    if env::var("TERM_PROGRAM").as_deref() == Ok("iTerm.app") {
        return true;
    }
    let term = env::var("TERM").unwrap_or_default();
    if term.contains("italic")
        || term.contains("256color")
        || term.contains("tmux")
        || term.contains("screen")
    {
        return true;
    }
    if let Some(host) = env::var("PSVersion").ok() {
        if host.starts_with('7') || host.starts_with('8') || host.starts_with('9') {
            return true;
        }
    }
    false
}

/// ASCII art for "ANGLES CODE" in a slanted font.
/// Each row is the concatenation of two letter blocks: ANgles | CODE
const ANGLES_CODE_ART: &[&str] = &[
    "  /$$      /$$          /$$$$$$$                /$$",
    " | $$  /$ | $$         | $$__  $$              | $$",
    " | $$ /$$$| $$  /$$$$$$| $$  \\ $$   /$$$$$$   /$$$$$$   /$$   /$$",
    " | $$/$$ $$ $$ /$$__  $$| $$$$$$$  /$$__  $$ |_  $$_/  | $$  | $$",
    " | $$$$_  $$$$| $$  \\ $$| $$__  $$| $$  \\ $$   | $$    | $$  | $$",
    " | $$$/ \\  $$$| $$  | $$| $$  \\ $$| $$  | $$   | $$ /$$| $$  | $$",
    " | $$/   \\  $$|  $$$$$$/| $$$$$$$/|  $$$$$$/   |  $$$$/|  $$$$$$/",
    " |__/     \\__/  \\______/ |___/___/  \\______/     \\___/   \\______/ ",
    "",
    "  α  Angles Code CLI",
    "  Terminal-based agentic coding assistant",
];

pub fn print() {
    let italic = supports_italic();
    let start = if italic { "\x1b[3m" } else { "" };
    let end = if italic { "\x1b[0m" } else { "" };
    let b = "\x1b[1m"; // bold subtext
    let accent = "\x1b[38;2;90;200;255m"; // angles-blue
    let reset = "\x1b[0m";

    for line in ANGLES_CODE_ART.iter() {
        if line.is_empty() {
            println!();
            continue;
        }
        let colored = if line.starts_with(' ') && !line.contains('_') {
            format!("{}{}{}", b, line, reset)
        } else {
            format!("{}{}{}{}", accent, start, line, reset)
        };
        println!("{}", colored);
    }
    println!();
    io::stdout().flush().ok();
}
