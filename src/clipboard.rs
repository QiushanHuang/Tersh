use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::io::Write;

pub fn osc52_sequence(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", STANDARD.encode(text.as_bytes()))
}

pub fn write_clipboard<W: Write>(writer: &mut W, text: &str) -> Result<()> {
    writer.write_all(osc52_sequence(text).as_bytes())?;
    writer.flush()?;
    Ok(())
}
