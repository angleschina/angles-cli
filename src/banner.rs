/// Angles Code CLI — ASCII Art Banner with italic support
///
/// Prints a large "ANGLES" banner at startup, using terminal escape sequences
/// for italic text when supported, falling back to bold/plain on older terminals.

use std::env;
use std::io::{self, Write};

/// Check if the current terminal supports italic text.
/// Returns true for:
/// - Windows Terminal (WT_SESSION env set)
/// - iTerm2 / Terminal.app on macOS
/// - tmux/screen with italic capabilities
/// - TERM values containing 'italic' or known capable variants
fn supports_italic() -> bool {
    // Windows Terminal sets WT_SESSION
    if cfg!(windows) && env::var("WT_SESSION").is_ok() {
        return true;
    }

    // iTerm2 / modern terminals
    if env::var("TERM_PROGRAM").as_deref() == Ok("iTerm.app") {
        return true;
    }

    // tmux/screen/... often support italic via terminfo
    let term = env::var("TERM").unwrap_or_default();
    if term.contains("italic")
        || term.contains("256color")
        || term.contains("tmux")
        || term.contains("screen")
    {
        return true;
    }

    // PowerShell 7+ on Windows
    if let Some(host) = env::var("PSVersion").ok() {
        // PSVersion >= 7 means pwsh, which supports italics
        if host.starts_with('7') || host.starts_with('8') || host.starts_with('9') {
            return true;
        }
    }

    false
}

/// ASCII art for "ANGLES" in a slanted/italic-friendly block font.
/// Each line is pre-formatted; we'll wrap with ANSI escapes later.
const ANGLES_ART: &[&str] = &[
    "  ____             _       _       ",
    " |  _ \\ _____      _| | __ _| |_ ___ ",
    " | |_) / _ \\ \\ /\\ / / |/ _` | __/ _ \\",
    " |  __/ (_) \\ V  V /| | (_| | ||  __/",
    " |_|   \\___/ \\_/\\_/ |_|\\__,_|\\__\\___|",
    "",
    "  α  Angles Code CLI",
    "  Terminal-based agentic coding assistant",
];

pub fn print() {
    let italic = supports_italic();
    let bold = true; // bold is widely supported

    let start = if italic { "\x1b[3m" } else { "" };
    let end = if italic { "\x1b[0m" } else { "" };
    let b = if bold { "\x1b[1m" } else { "" };
    let accent = "\x1b[38;2;90;200;255m"; // angles-blue from install.ps1
    let reset = "\x1b[0m";

    // Print banner
    for line in ANGLES_ART.iter() {
        if line.is_empty() {
            println!();
            continue;
        }
        // Wrap with colors: blue accent for the block, bold for subtext
        let colored = if line.starts_with(' ') && !line.contains('_') {
            // Subtext line
            format!("{}{}{}", b, line, reset)
        } else {
            // ASCII art line
            format!("{}{}{}{}", accent, start, line, reset)
        };
        println!("{}", colored);
    }
    println!();

    // Flush to ensure banner appears before any other output
    io::stdout().flush().ok();
}
