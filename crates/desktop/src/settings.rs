use iced::{Theme, theme::Mode};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Settings {
    pub theme: ThemePreference,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    #[default]
    System,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::System];

    pub fn theme(self, system: Mode) -> Theme {
        match self {
            Self::Light => Theme::Light,
            Self::Dark => Theme::Dark,
            Self::System => match system {
                Mode::Dark => Theme::Dark,
                Mode::Light | Mode::None => Theme::Light,
            },
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
}

impl std::str::FromStr for ThemePreference {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "system" => Ok(Self::System),
            _ => Err(format!("Unknown theme preference: {value}")),
        }
    }
}

impl std::fmt::Display for ThemePreference {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::System => "System",
        })
    }
}
