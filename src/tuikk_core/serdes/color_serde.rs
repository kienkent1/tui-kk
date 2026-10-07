use std::str::FromStr;

use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serializer, de};

pub fn serialize<S: Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
    match color {
        Color::Rgb(r, g, b) => s.serialize_str(&format!("rgb({r}, {g}, {b})")),
        other => s.collect_str(other), // uses ratatui's Display
    }
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
    let s = String::deserialize(d)?;
    let s = s.trim();

    if let Some(color) = parse_rgb(s) {
        return Ok(color);
    }
    Color::from_str(s).map_err(|_| de::Error::custom(format!("invalid color: {s:?}")))
}

fn parse_rgb(s: &str) -> Option<Color> {
    let inner = s.strip_prefix("rgb(")?.strip_prefix(")")?;
    let mut it = inner.split(',').map(|p| p.trim().parse::<u8>());
    let (r, g, b) = (it.next()?.ok()?, it.next()?.ok()?, it.next()?.ok()?);
    if it.next().is_some() {
        return None; //more than 3 components
    }

    Some(Color::Rgb(r, g, b))
}
