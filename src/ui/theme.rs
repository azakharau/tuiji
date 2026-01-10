use ratatui::style::Color;

use crate::config::{AppConfig, CustomThemeConfig};

#[derive(Clone, Copy, Debug)]
pub struct ThemePalette {
    pub background: Color,
    pub text: Color,
    pub accent: Color,
    pub selection: Color,
    pub border: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub success: Color,
    pub logo: Color,
    pub mode_normal_bg: Color,
    pub mode_insert_bg: Color,
    pub mode_visual_bg: Color,
    pub mode_command_bg: Color,
    pub mode_text: Color,
}

#[derive(Clone, Debug)]
pub struct ThemeInfo {
    pub id: String,
    pub name: String,
}

pub struct ThemeRegistry;

impl ThemeRegistry {
    pub fn get(theme_id: &str) -> ThemePalette {
        match Self::resolve_id(theme_id) {
            "tokyonight" => ThemePalette {
                background: Color::Rgb(0x1a, 0x1b, 0x26),
                text: Color::Rgb(0xc0, 0xca, 0xf5),
                accent: Color::Rgb(0x7a, 0xa2, 0xf7),
                selection: Color::Rgb(0x33, 0x46, 0x7c),
                border: Color::Rgb(0x56, 0x5f, 0x89),
                error: Color::Rgb(0xf7, 0x76, 0x8e),
                warning: Color::Rgb(0xe0, 0xaf, 0x68),
                info: Color::Rgb(0x7d, 0xcf, 0xff),
                success: Color::Rgb(0x9e, 0xce, 0x6a),
                logo: Color::White,
                ..mode_colors()
            },
            "solarized_dark" => ThemePalette {
                background: Color::Rgb(0x00, 0x2b, 0x36),
                text: Color::Rgb(0x93, 0xa1, 0xa1),
                accent: Color::Rgb(0x26, 0x8b, 0xd2),
                selection: Color::Rgb(0x07, 0x36, 0x42),
                border: Color::Rgb(0x58, 0x6e, 0x75),
                error: Color::Rgb(0xdc, 0x32, 0x2f),
                warning: Color::Rgb(0xb5, 0x89, 0x00),
                info: Color::Rgb(0x2a, 0xa1, 0x98),
                success: Color::Rgb(0x85, 0x99, 0x00),
                logo: Color::White,
                ..mode_colors()
            },
            _ => ThemePalette {
                background: Color::Black,
                text: Color::White,
                accent: Color::Blue,
                selection: Color::DarkGray,
                border: Color::Gray,
                error: Color::Red,
                warning: Color::Yellow,
                info: Color::Cyan,
                success: Color::Green,
                logo: Color::White,
                ..mode_colors()
            },
        }
    }

    pub fn resolve_id(theme_id: &str) -> &'static str {
        if theme_id.eq_ignore_ascii_case("tokyonight") {
            "tokyonight"
        } else if theme_id.eq_ignore_ascii_case("solarized_dark") {
            "solarized_dark"
        } else if theme_id.eq_ignore_ascii_case("default")
            || theme_id.eq_ignore_ascii_case("dark")
        {
            "default"
        } else {
            "default"
        }
    }

    pub fn is_builtin_id(theme_id: &str) -> bool {
        matches!(
            theme_id.to_lowercase().as_str(),
            "default" | "dark" | "tokyonight" | "solarized_dark"
        )
    }

    pub fn themes() -> Vec<ThemeInfo> {
        vec![
            ThemeInfo {
                id: "default".to_string(),
                name: "Default".to_string(),
            },
            ThemeInfo {
                id: "tokyonight".to_string(),
                name: "Tokyonight".to_string(),
            },
            ThemeInfo {
                id: "solarized_dark".to_string(),
                name: "Solarized Dark".to_string(),
            },
        ]
    }

    pub fn themes_with_custom(custom: &[CustomThemeConfig]) -> Vec<ThemeInfo> {
        let mut themes = Self::themes();
        themes.extend(custom.iter().map(|theme| ThemeInfo {
            id: theme.id.clone(),
            name: theme.name.clone(),
        }));
        themes
    }

    pub fn default_id() -> &'static str {
        "default"
    }

    pub fn palette_from_config(cfg: &AppConfig, theme_id: &str) -> ThemePalette {
        if let Some(custom) = cfg.ui.custom_themes.iter().find(|t| t.id == theme_id) {
            if let Some(palette) = Self::custom_palette(custom) {
                return palette;
            }
        }
        Self::get(theme_id)
    }

    pub fn custom_palette(theme: &CustomThemeConfig) -> Option<ThemePalette> {
        Some(ThemePalette {
            background: parse_hex_color(&theme.palette.background)?,
            text: parse_hex_color(&theme.palette.text)?,
            accent: parse_hex_color(&theme.palette.accent)?,
            selection: parse_hex_color(&theme.palette.selection)?,
            border: parse_hex_color(&theme.palette.border)?,
            error: parse_hex_color(&theme.palette.error)?,
            warning: parse_hex_color(&theme.palette.warning)?,
            info: parse_hex_color(&theme.palette.info)?,
            success: parse_hex_color(&theme.palette.success)?,
            logo: Color::White,
            ..mode_colors()
        })
    }
}

fn mode_colors() -> ThemePalette {
    ThemePalette {
        background: Color::Black,
        text: Color::White,
        accent: Color::Blue,
        selection: Color::DarkGray,
        border: Color::Gray,
        error: Color::Red,
        warning: Color::Yellow,
        info: Color::Cyan,
        success: Color::Green,
        logo: Color::White,
        mode_normal_bg: Color::Blue,
        mode_insert_bg: Color::LightGreen,
        mode_visual_bg: Color::LightMagenta,
        mode_command_bg: Color::Yellow,
        mode_text: Color::Black,
    }
}

pub fn color_to_hex(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
        Color::Black => "#000000".to_string(),
        Color::White => "#ffffff".to_string(),
        Color::Red => "#ff0000".to_string(),
        Color::Green => "#00ff00".to_string(),
        Color::Blue => "#0000ff".to_string(),
        Color::Yellow => "#ffff00".to_string(),
        Color::Cyan => "#00ffff".to_string(),
        Color::Magenta => "#ff00ff".to_string(),
        Color::Gray => "#808080".to_string(),
        Color::DarkGray => "#404040".to_string(),
        _ => "#000000".to_string(),
    }
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let trimmed = value.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&trimmed[0..2], 16).ok()?;
    let g = u8::from_str_radix(&trimmed[2..4], 16).ok()?;
    let b = u8::from_str_radix(&trimmed[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}
