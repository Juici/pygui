use pyo3::prelude::*;
use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;

use std::collections::VecDeque;
use piston_window::*;

type Scalar = f64;
type Point = (Scalar, Scalar);

/// Represents an rgb color, with optional alpha.
pub struct Color(i32, i32, i32, Option<f32>);

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        const MIN: i32 = 0;
        const MAX: i32 = 255;

        let r = self.0.min(MAX).max(MIN);
        let g = self.1.min(MAX).max(MIN);
        let b = self.2.min(MAX).max(MIN);
        let a = self.3.unwrap_or(1.0).max(0.0).min(1.0);

        [
            r as f32 / MAX as f32,
            g as f32 / MAX as f32,
            b as f32 / MAX as f32,
            a,
        ]
    }
}

impl<'a> FromPyObject<'a> for Color {
    fn extract(ob: &'a PyObjectRef) -> PyResult<Self> {
        let t = PyTuple::try_from(ob)?;
        let slice = t.as_slice();

        if t.len() != 3 && t.len() != 4 {
            return Err(exc::ValueError::new("Expected color (r, g, b[, a])"));
        }

        let r: i32 = slice[0].extract::<i32>(ob.py())?;
        let g: i32 = slice[1].extract::<i32>(ob.py())?;
        let b: i32 = slice[2].extract::<i32>(ob.py())?;
        let a: Option<f32> = if t.len() == 4 {
            Some(slice[3].extract::<f32>(ob.py())?)
        } else {
            None
        };

        Ok(Color(r, g, b, a))
    }
}

pub enum DrawAction {
    Clear(Color),
    Point(Point, Color),
    Image,
    Circle {
        center: Point,
        radius: Scalar,
        line_color: Color,
        line_width: Option<Scalar>,
        fill_color: Option<Color>,
    },
    Arc {
        center: Point,
        radius: Scalar,
        line_color: Color,
        line_width: Option<Scalar>,
        bounds: (Scalar, Scalar),
    },
    Polygon {
        points: Vec<Point>,
        line_color: Color,
        line_width: Option<Scalar>,
    },
    Polyline,
    Line,
    Text,
}

/// Represents the drawable area of a frame.
#[pyclass]
pub struct Canvas {
    draw_queue: VecDeque<DrawAction>,
    size: (u32, u32),
    token: PyToken,
}

impl Canvas {
    /// Constructs a new canvas.
    pub fn new<'p>(py: &'p Python) -> Py<Canvas> {
        py.init(|token| Canvas {
            draw_queue: VecDeque::new(),
            size: (0, 0),
            token,
        }).unwrap()
    }

    /// Updates the canvas size.
    pub fn update_size(&mut self, size: (u32, u32)) {
        self.size = size;
    }

    /// Draws the `draw_queue` to the graphics context.
    pub fn draw_canvas(&mut self, c: &Context, g: &mut G2d) {
        clear([0.0, 0.0, 0.0, 1.0], g);

        while let Some(d) = self.draw_queue.pop_front() {
            match d {
                DrawAction::Clear(color) => clear(color.into(), g),
                DrawAction::Point(point, color) => {
                    let square = rectangle::square(point.0, point.1, 1.0);
                    Rectangle::new(color.into())
                        .draw(square, &Default::default(), c.transform, g);
                }
                DrawAction::Image => (), // TODO
                DrawAction::Circle {
                    center, radius, line_width, line_color, fill_color
                } => {
                    let mut ellipse =
                        Ellipse::new_border(line_color.into(), line_width.unwrap_or(1.0));
                    if let Some(fill_color) = fill_color {
                        ellipse = ellipse.color(fill_color.into());
                    }

                    let circle = ellipse::circle(center.0, center.1, radius);
                    ellipse.draw(circle, &Default::default(), c.transform, g);
                }
                DrawAction::Arc {
                    center, radius, line_width, line_color, bounds
                } => {
                    let circle = ellipse::circle(center.0, center.1, radius);
                    CircleArc::new(line_color.into(), line_width.unwrap_or(1.0), bounds.0, bounds.1)
                        .draw(circle, &Default::default(), c.transform, g);
                }
                _ => (),
            }
        }
    }
}

#[pymethods]
impl Canvas {
    /// Gets the size of the canvas.
    pub fn get_size(&self) -> PyResult<(u32, u32)> {
        Ok(self.size)
    }

    /// Clears the canvas.
    pub fn clear(&mut self, color: Color) -> PyResult<()> {
        self.draw_queue.push_back(DrawAction::Clear(color));
        Ok(())
    }

    /// Draws a point on the canvas.
    pub fn draw_point(&mut self, point: Point, color: Color) -> PyResult<()> {
        self.draw_queue.push_back(DrawAction::Point(point, color));
        Ok(())
    }

    /// Draws a circle on the canvas.
    pub fn draw_circle(&mut self,
                       center: Point,
                       radius: Scalar,
                       line_color: Color,
                       line_width: Option<Scalar>,
                       fill_color: Option<Color>) -> PyResult<()> {
        self.draw_queue.push_back(DrawAction::Circle {
            center,
            radius,
            line_width,
            line_color,
            fill_color,
        });
        Ok(())
    }

    /// Draws an arc on the canvas.
    pub fn draw_arc(&mut self,
                    center: Point,
                    radius: Scalar,
                    bounds: (Scalar, Scalar),
                    line_color: Color,
                    line_width: Option<Scalar>) -> PyResult<()> {
        self.draw_queue.push_back(DrawAction::Arc {
            center,
            radius,
            line_width,
            line_color,
            bounds,
        });
        Ok(())
    }
}