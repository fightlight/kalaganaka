use super::calendar::layout;
use crate::app::{App, Message};
use iced::{
    Element, Fill,
    widget::{column, container, row, rule, scrollable, text},
};
use tracking_core::statistics;

pub fn view(app: &App) -> Element<'_, Message> {
    let mut content = column![
        text("Time by task").size(26),
        text("Completed frames · selected calendar day/week")
    ]
    .spacing(14);
    if let Some((start, end)) = layout::period(app.date, app.mode) {
        let totals = statistics::totals(&app.snapshot, start, end);
        let total_seconds = totals
            .iter()
            .map(|total| total.duration.num_seconds())
            .sum();
        content = content
            .push(
                row![
                    text(format!(
                        "{} — {}",
                        start.with_timezone(&chrono::Local).format("%d %b %Y"),
                        (end - chrono::Duration::seconds(1))
                            .with_timezone(&chrono::Local)
                            .format("%d %b %Y")
                    ))
                    .width(Fill),
                    text(format_duration(total_seconds))
                ]
                .spacing(20),
            )
            .push(rule::horizontal(1));
        if totals.is_empty() {
            content = content.push(text("Add a task and record some time to get started."));
        }
        for total in totals {
            let seconds = total.duration.num_seconds();
            content = content.push(
                row![text(total.name).width(Fill), text(format_duration(seconds))].spacing(20),
            );
        }
    }
    container(scrollable(content))
        .padding(24)
        .width(Fill)
        .height(Fill)
        .into()
}

fn format_duration(seconds: i64) -> String {
    format!(
        "{}h {:02}m {:02}s",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}
