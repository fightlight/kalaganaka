use iced::{
    Color, Element, Point, Rectangle, Renderer, Size, Theme, mouse,
    widget::{
        canvas,
        canvas::{Frame, Geometry, Path, Stroke},
    },
};

#[derive(Clone, Copy)]
pub enum Icon {
    Play(Color),
    Pause(Color),
    Calendar,
    Previous,
    Next,
    Settings,
    Statistics,
    Pencil,
    Check,
    Add,
}

pub fn view<'a, Message: 'a>(icon: Icon, size: f32) -> Element<'a, Message> {
    canvas(icon).width(size).height(size).into()
}

impl<Message> canvas::Program<Message> for Icon {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        frame.scale(bounds.width.min(bounds.height) / 24.0);
        let palette = theme.extended_palette();
        let colour = match self {
            Self::Play(colour) | Self::Pause(colour) => *colour,
            Self::Calendar
            | Self::Previous
            | Self::Next
            | Self::Settings
            | Self::Statistics
            | Self::Pencil
            | Self::Check
            | Self::Add => palette.background.base.text,
        };
        let stroke = Stroke::default().with_color(colour).with_width(1.8);
        match self {
            Self::Add => {
                frame.stroke(&Path::circle(Point::new(12.0, 12.0), 9.0), stroke);
                frame.stroke(
                    &Path::line(Point::new(7.0, 12.0), Point::new(17.0, 12.0)),
                    stroke,
                );
                frame.stroke(
                    &Path::line(Point::new(12.0, 7.0), Point::new(12.0, 17.0)),
                    stroke,
                );
            }
            Self::Play(_) => {
                let triangle = Path::new(|path| {
                    path.move_to(Point::new(7.0, 3.0));
                    path.line_to(Point::new(21.0, 12.0));
                    path.line_to(Point::new(7.0, 21.0));
                    path.close();
                });
                frame.fill(&triangle, colour);
            }
            Self::Pause(_) => {
                frame.fill_rectangle(Point::new(5.0, 3.0), Size::new(5.0, 18.0), colour);
                frame.fill_rectangle(Point::new(14.0, 3.0), Size::new(5.0, 18.0), colour);
            }
            Self::Calendar => {
                frame.stroke(
                    &Path::rectangle(Point::new(3.0, 5.0), Size::new(18.0, 16.0)),
                    stroke,
                );
                frame.stroke(
                    &Path::line(Point::new(3.0, 10.0), Point::new(21.0, 10.0)),
                    stroke,
                );
                for x in [7.0, 17.0] {
                    frame.stroke(&Path::line(Point::new(x, 2.0), Point::new(x, 7.0)), stroke);
                }
                for y in [13.0, 17.0] {
                    for x in [6.0, 11.0, 16.0] {
                        frame.fill_rectangle(Point::new(x, y), Size::new(2.0, 2.0), colour);
                    }
                }
            }
            Self::Previous | Self::Next => {
                frame.stroke(&Path::circle(Point::new(12.0, 12.0), 9.0), stroke);
                let (outer, inner) = if matches!(self, Self::Previous) {
                    (14.0, 10.0)
                } else {
                    (10.0, 14.0)
                };
                let chevron = Path::new(|path| {
                    path.move_to(Point::new(outer, 8.0));
                    path.line_to(Point::new(inner, 12.0));
                    path.line_to(Point::new(outer, 16.0));
                });
                frame.stroke(&chevron, stroke);
            }
            Self::Settings => {
                let gear = Path::new(|path| {
                    for index in 0..32 {
                        let angle = index as f32 * std::f32::consts::TAU / 32.0;
                        let radius = if index % 4 == 1 || index % 4 == 2 {
                            10.0
                        } else {
                            7.5
                        };
                        let point =
                            Point::new(12.0 + radius * angle.cos(), 12.0 + radius * angle.sin());
                        if index == 0 {
                            path.move_to(point);
                        } else {
                            path.line_to(point);
                        }
                    }
                    path.close();
                });
                frame.stroke(&gear, stroke);
                frame.stroke(&Path::circle(Point::new(12.0, 12.0), 3.0), stroke);
            }
            Self::Statistics => {
                let axes = Path::new(|path| {
                    path.move_to(Point::new(3.0, 3.0));
                    path.line_to(Point::new(3.0, 21.0));
                    path.line_to(Point::new(22.0, 21.0));
                });
                frame.stroke(&axes, stroke);
                for (x, height) in [(6.0, 7.0), (11.5, 12.0), (17.0, 17.0)] {
                    frame.fill_rectangle(
                        Point::new(x, 20.0 - height),
                        Size::new(3.5, height),
                        colour,
                    );
                }
            }
            Self::Check => {
                let check = Path::new(|path| {
                    path.move_to(Point::new(4.0, 12.0));
                    path.line_to(Point::new(9.0, 17.0));
                    path.line_to(Point::new(20.0, 6.0));
                });
                frame.stroke(&check, stroke);
            }
            Self::Pencil => {
                let pencil = Path::new(|path| {
                    path.move_to(Point::new(4.0, 20.0));
                    path.line_to(Point::new(5.0, 15.0));
                    path.line_to(Point::new(17.0, 3.0));
                    path.line_to(Point::new(21.0, 7.0));
                    path.line_to(Point::new(9.0, 19.0));
                    path.close();
                });
                frame.stroke(&pencil, stroke);
            }
        }
        vec![frame.into_geometry()]
    }
}
