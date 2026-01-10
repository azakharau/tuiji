use crate::{
    app::state::Mode,
    config::AppConfig,
    ui::theme::{ThemePalette, ThemeRegistry},
};

#[derive(Clone, Debug)]
pub struct RenderContext {
    theme_id: String,
    palette: ThemePalette,
    mode: Mode,
}

impl RenderContext {
    pub fn new(mode: Mode) -> Self {
        let theme_id = ThemeRegistry::default_id().to_string();
        let palette = ThemeRegistry::get(theme_id.as_str());
        Self {
            theme_id,
            palette,
            mode,
        }
    }

    pub fn from_config(cfg: &AppConfig, mode: Mode) -> Self {
        let theme_id = cfg.ui.theme.clone();
        let palette = ThemeRegistry::palette_from_config(cfg, theme_id.as_str());
        Self {
            theme_id,
            palette,
            mode,
        }
    }

    pub fn colors(&self) -> &ThemePalette {
        &self.palette
    }

    pub fn theme_id(&self) -> &str {
        self.theme_id.as_str()
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
}
