use iced::{Subscription, futures::StreamExt, theme::Mode};
use mundy::{ColorScheme, Interest, Preferences};

/// Read native preferences independently of Iced's per-window theme state.
/// The stream emits the initial appearance followed by OS preference changes.
pub fn changes() -> Subscription<Mode> {
    Subscription::run(|| {
        Preferences::stream(Interest::ColorScheme).map(|preferences| {
            match preferences.color_scheme {
                ColorScheme::Dark => Mode::Dark,
                ColorScheme::Light => Mode::Light,
                ColorScheme::NoPreference => Mode::None,
            }
        })
    })
}
