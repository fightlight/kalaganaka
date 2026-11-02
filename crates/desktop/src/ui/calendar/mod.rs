mod canvas;
pub mod layout;

use crate::app::{App, Message};
use iced::{
    Element, Fill,
    widget::{canvas as canvas_widget, scrollable},
};

pub fn view(app: &App) -> Element<'_, Message> {
    scrollable(
        canvas_widget(canvas::Calendar { app })
            .width(Fill)
            .height(layout::HEIGHT),
    )
    .height(Fill)
    .into()
}
