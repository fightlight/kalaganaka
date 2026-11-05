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
    Settings,
    Statistics,
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
            Self::Settings | Self::Statistics => palette.background.base.text,
        };
        let stroke = Stroke::default().with_color(colour).with_width(1.8);
        match self {
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
        }
        vec![frame.into_geometry()]
    }
}
