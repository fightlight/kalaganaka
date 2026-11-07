pub mod appearance;
pub mod paths;

pub fn window_settings() -> iced::window::settings::PlatformSpecific {
    #[cfg(target_os = "linux")]
    {
        iced::window::settings::PlatformSpecific {
            application_id: "kalaganaka".into(),
            ..Default::default()
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Default::default()
    }
}
