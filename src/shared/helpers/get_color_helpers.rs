use ratatui::style::Color;

use crate::tuikk_core::themes::ThemeColor;

#[derive(Clone, Copy)]
pub struct BtnPalette {
    pub fg: Color,
    pub fg_hover: Color,
    pub fg_press: Color,
}

impl ThemeColor {
    pub fn btn_start(&self) -> BtnPalette {
        BtnPalette {
            fg: self.btn_start_fg,
            fg_hover: self.btn_start_fg_hover,
            fg_press: self.btn_start_fg_press,
        }
    }

    pub fn btn_stop(&self) -> BtnPalette {
        BtnPalette {
            fg: self.btn_stop_fg,
            fg_hover: self.btn_stop_fg_hover,
            fg_press: self.btn_stop_fg_press,
        }
    }

    pub fn btn_remove(&self) -> BtnPalette {
        BtnPalette {
            fg: self.btn_remove_fg,
            fg_hover: self.btn_remove_fg_hover,
            fg_press: self.btn_remove_fg_press,
        }
    }
}
