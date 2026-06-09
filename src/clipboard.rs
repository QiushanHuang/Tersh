use anyhow::{Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::io::Write;

pub const MAX_OSC52_INPUT_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardMode {
    Osc52,
    Off,
}

impl ClipboardMode {
    pub fn from_env() -> Self {
        match std::env::var("TERSH_CLIPBOARD") {
            Ok(value) if value.eq_ignore_ascii_case("off") => Self::Off,
            _ => Self::Osc52,
        }
    }
}

pub fn osc52_sequence(text: &str) -> Result<String> {
    if text.len() > MAX_OSC52_INPUT_BYTES {
        bail!(
            "clipboard payload too large: {} bytes exceeds {} bytes",
            text.len(),
            MAX_OSC52_INPUT_BYTES
        );
    }
    Ok(format!(
        "\x1b]52;c;{}\x07",
        STANDARD.encode(text.as_bytes())
    ))
}

pub fn write_clipboard<W: Write>(writer: &mut W, text: &str) -> Result<()> {
    write_clipboard_with_mode(writer, text, ClipboardMode::from_env()).map(|_| ())
}

pub fn write_clipboard_with_mode<W: Write>(
    writer: &mut W,
    text: &str,
    mode: ClipboardMode,
) -> Result<bool> {
    match mode {
        ClipboardMode::Off => Ok(false),
        ClipboardMode::Osc52 => {
            writer.write_all(osc52_sequence(text)?.as_bytes())?;
            writer.flush()?;
            Ok(true)
        }
    }
}
