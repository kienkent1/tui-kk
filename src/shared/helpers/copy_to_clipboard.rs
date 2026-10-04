use base64::{Engine, engine::general_purpose::STANDARD};
use std::io::Write;
pub fn copy_to_clipboard(text: &str) -> std::io::Result<()> {
    let b64 = STANDARD.encode(text);
    let mut out = std::io::stdout();
    write!(out, "\x1b]52;c;{b64}\x07")?;
    out.flush()
}
