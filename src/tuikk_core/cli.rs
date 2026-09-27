use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long, value_parser = parse_rate)]
    pub frame_rate: Option<f64>,

    #[arg(short, long, value_parser = parse_rate)]
    pub tick_rate: Option<f64>,
}

pub const RATE_RANGE: std::ops::RangeInclusive<f64> = 1.0..=240.0;

fn parse_rate(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| format!("`{s}` is not a number "))?;
    if RATE_RANGE.contains(&v) {
        // NaN / inf / 0 / negative numbers are not within the range.
        Ok(v)
    } else {
        Err(format!(
            "Must fall within the range of {}–{}",
            RATE_RANGE.start(),
            RATE_RANGE.end()
        ))
    }
}
