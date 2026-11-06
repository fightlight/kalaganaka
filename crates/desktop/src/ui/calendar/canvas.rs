use super::layout;
use crate::app::{App, FrameSelection, Message};
use chrono::{Days, Utc};
use iced::{
    Color, Point, Rectangle, Renderer, Size, Theme, mouse,
    widget::canvas::{self, Frame, Geometry, Path, Stroke, Text},
};

pub struct Calendar<'a> {
    pub app: &'a App,
}

impl canvas::Program<Message> for Calendar<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if !matches!(
            event,
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) {
            return None;
        }
        if !self.app.ready() {
            return None;
        }
        self.hit_frame(bounds, cursor)
            .map(|frame| canvas::Action::publish(Message::SelectFrame(frame)).and_capture())
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.app.ready() && self.hit_frame(bounds, cursor).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let dates = layout::dates(self.app.date, self.app.mode);
        let width = ((bounds.width - layout::GUTTER) / dates.len() as f32).max(1.0);
        let palette = theme.extended_palette();
        let grid = palette.background.strong.color;
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), palette.background.base.color);
        for hour in 0..24 {
            let y = layout::y(hour as f32 * 60.0);
            frame.stroke(
                &Path::line(Point::new(layout::GUTTER, y), Point::new(bounds.width, y)),
                Stroke::default().with_color(grid),
            );
            frame.fill_text(Text {
                content: format!("{hour:02}:00"),
                position: Point::new(4.0, y + 3.0),
                size: 12.0.into(),
                color: palette.background.base.text,
                ..Default::default()
            });
        }
        for (index, date) in dates.iter().enumerate() {
            let x = layout::GUTTER + index as f32 * width;
            frame.stroke(
                &Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
                Stroke::default().with_color(grid),
            );
            frame.fill_text(Text {
                content: date.format("%a %d %b").to_string(),
                position: Point::new(x + 6.0, 7.0),
                size: 14.0.into(),
                color: palette.background.base.text,
                ..Default::default()
            });
            let Some(start) = layout::day_start(*date) else {
                continue;
            };
            let Some(end) = date
                .checked_add_days(Days::new(1))
                .and_then(layout::day_start)
            else {
                continue;
            };
            for recorded in &self.app.snapshot.frames {
                self.draw_block(
                    &mut frame,
                    theme,
                    x,
                    width,
                    start,
                    end,
                    recorded.task_id,
                    recorded.started_at,
                    recorded.ended_at,
                    false,
                );
            }
            if let Some(active) = &self.app.snapshot.active {
                self.draw_block(
                    &mut frame,
                    theme,
                    x,
                    width,
                    start,
                    end,
                    active.task_id,
                    active.started_at,
                    self.app.now.with_timezone(&Utc),
                    true,
                );
            }
            if *date == self.app.now.date_naive() {
                let y = layout::y(layout::minute(self.app.now.with_timezone(&Utc)));
                frame.stroke(
                    &Path::line(Point::new(x, y), Point::new(x + width, y)),
                    Stroke::default()
                        .with_color(Color::from_rgb8(215, 45, 45))
                        .with_width(2.0),
                );
            }
        }
        vec![frame.into_geometry()]
    }
}

impl Calendar<'_> {
    fn hit_frame(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<FrameSelection> {
        let point = cursor.position_in(bounds)?;
        let dates = layout::dates(self.app.date, self.app.mode);
        let width = ((bounds.width - layout::GUTTER) / dates.len() as f32).max(1.0);
        let index = ((point.x - layout::GUTTER) / width).floor() as usize;
        if point.x < layout::GUTTER || index >= dates.len() {
            return None;
        }
        let x = layout::GUTTER + index as f32 * width;
        if point.x < x + 3.0 || point.x > x + 3.0 + (width - 6.0).max(1.0) {
            return None;
        }
        let start = layout::day_start(dates[index])?;
        let end = dates[index]
            .checked_add_days(Days::new(1))
            .and_then(layout::day_start)?;
        let contains = |from: chrono::DateTime<Utc>, to: chrono::DateTime<Utc>| {
            let from = from.max(start);
            let to = to.min(end);
            if to <= from {
                return false;
            }
            let top = layout::y(layout::minute(from));
            let bottom = if to == end {
                layout::HEIGHT
            } else {
                layout::y(layout::minute(to))
            };
            point.y >= top && point.y <= top + (bottom - top).max(3.0)
        };
        if let Some(active) = &self.app.snapshot.active {
            if contains(active.started_at, self.app.now.with_timezone(&Utc)) {
                return Some(FrameSelection::Active(active.task_id, active.started_at));
            }
        }
        self.app
            .snapshot
            .frames
            .iter()
            .rev()
            .find(|frame| contains(frame.started_at, frame.ended_at))
            .map(|frame| FrameSelection::Completed(frame.id))
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_block(
        &self,
        frame: &mut Frame,
        theme: &Theme,
        x: f32,
        width: f32,
        day_start: chrono::DateTime<Utc>,
        day_end: chrono::DateTime<Utc>,
        task_id: uuid::Uuid,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
        running: bool,
    ) {
        let start = start.max(day_start);
        let end = end.min(day_end);
        if end <= start {
            return;
        }
        let Some(task) = self
            .app
            .snapshot
            .tasks
            .iter()
            .find(|task| task.id == task_id)
        else {
            return;
        };
        let top = layout::y(layout::minute(start));
        let bottom = if end == day_end {
            layout::HEIGHT
        } else {
            layout::y(layout::minute(end))
        };
        let height = (bottom - top).max(3.0);
        let colour = Color::from_rgb8(
            (task.colour >> 16) as u8,
            (task.colour >> 8) as u8,
            task.colour as u8,
        );
        let background = theme.palette().background;
        let opacity = if running { 0.55 } else { 1.0 };
        let blend = |component, base| component * opacity + base * (1.0 - opacity);
        let fill = Color::from_rgb(
            blend(colour.r, background.r),
            blend(colour.g, background.g),
            blend(colour.b, background.b),
        );
        let pair = iced::theme::palette::Pair::new(fill, theme.palette().text);
        frame.fill_rectangle(
            Point::new(x + 3.0, top),
            Size::new((width - 6.0).max(1.0), height),
            pair.color,
        );
        if height >= 16.0 {
            let max_chars = ((width - 16.0) / 8.0).max(0.0) as usize;
            frame.fill_text(Text {
                content: task.name.chars().take(max_chars).collect(),
                position: Point::new(x + 7.0, top + 2.0),
                size: 12.0.into(),
                color: pair.text,
                ..Default::default()
            });
        }
    }
}
