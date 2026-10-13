use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long, value_parser = parse_rate)]
    pub frame_rate: Option<f64>,

    #[arg(short, long, value_parser = parse_rate)]
    pub tick_rate: Option<f64>,

    #[arg(long)]
    pub icon_preview: bool,
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

pub fn print_icon_preview() {
    println!("Icon preview\n");
    println!("  ascii : {}  {}  {}", "[>]", "[x]", "[d]");
    println!("  nerd  : {}  {}  {}", "\u{f04b}", "\u{f04d}", "\u{f1f8}");
    println!();
    println!("If the 'nerd' row shows boxes or '?', install a Nerd Font:");
    println!("  https://www.nerdfonts.com/font-downloads");
    println!("  Fedora : sudo dnf copr enable che/nerd-fonts");
    println!("           sudo dnf install nerd-fonts-jetbrains-mono");
    println!("           (or copy to ~/.local/share/fonts, then `fc-cache -fv`)");
    println!("  macOS  : brew install --cask font-jetbrains-mono-nerd-font");
    println!("  Windows: install the .ttf files, then set the font in your terminal profile");
    println!();
    println!("Then enable it with one of:");
    println!("  icon_set = \"nerd\"   (in config file)");
}
