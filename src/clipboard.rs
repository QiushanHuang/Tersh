use anyhow::{Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::io::Write;

pub const MAX_OSC52_INPUT_BYTES: usize = 16 * 1024;

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
    writer.write_all(osc52_sequence(text)?.as_bytes())?;
    writer.flush()?;
    Ok(())
}
