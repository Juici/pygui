use pyo3::prelude::*;

use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;
use pyo3::py::proto as pyproto;

use piston_window::*;
use glfw_window::GlfwWindow;

use canvas::Canvas;

type FrameWindow = PistonWindow<GlfwWindow>;

/// A frame with a drawable canvas.
#[pyclass]
pub struct Frame {
    window: FrameWindow,
    canvas: Py<Canvas>,
    draw_handler: Option<PyObject>,
    event_handler: Option<PyObject>,
    started: bool,
    token: PyToken,
}

#[pymethods(gc)]
impl Frame {
    /// Starts the frame draw and event handlers.
    /// Blocks the thread.
    pub fn start(&mut self) -> PyResult<()> {
        if self.started {
            // This shouldn't be possible.
            return Err(exc::RuntimeError::new("The frame can only be started once"));
        }
        self.started = true;

        while let Some(e) = self.window.next() {
            if let Some(_) = e.render_args() {
                let gil = Python::acquire_gil();

                let mut canvas = &self.canvas;
                {
                    // Update the canvas size.
                    let py = gil.python();
                    canvas.as_mut(py).update_size(self.window.size().into());
                }

                if let Some(ref handler) = self.draw_handler {
                    // Call the draw handler.
                    let py = gil.python();
                    let args = PyTuple::new(py, &[canvas]);
                    handler.call(py, args, NoArgs).unwrap();
                }

                self.window.draw_2d(&e, |c, g| {
                    // Draw the canvas.
                    let py = gil.python();
                    let canvas = canvas.as_mut(py);
                    canvas.draw_canvas(&c, g)
                });
            }
        }

        Ok(())
    }

    // Event handlers.

    /// Sets the draw handler for the frame.
    pub fn set_draw_handler(&mut self, handler: PyObject) -> PyResult<()> {
        self.draw_handler = Some(handler);
        Ok(())
    }

    /// Sets the event handler for the frame.
    pub fn set_event_handler(&mut self, handler: PyObject) -> PyResult<()> {
        self.event_handler = Some(handler);
        Ok(())
    }

    // Window functions.

    /// Shows the window.
    pub fn show(&mut self) -> PyResult<()> {
        self.window.show();
        Ok(())
    }

    /// Shows the window.
    pub fn hide(&mut self) -> PyResult<()> {
        self.window.hide();
        Ok(())
    }

    /// Shows the window.
    pub fn close(&mut self) -> PyResult<()> {
        self.window.set_should_close(true);
        Ok(())
    }

    /// Returns the frame title.
    pub fn get_title(&self) -> PyResult<String> {
        Ok(self.window.get_title())
    }

    /// Sets the frame title.
    pub fn set_title(&mut self, title: String) -> PyResult<()> {
        self.window.set_title(title);
        Ok(())
    }

    /// Returns the frame size.
    pub fn get_size(&self) -> PyResult<(u32, u32)> {
        let size: (u32, u32) = self.window.size().into();
        Ok(size)
    }

    /// Sets the frame size.
    pub fn set_size(&mut self, size: (i32, i32)) -> PyResult<()> {
        assert_pyval!(size.0 > 0, "Width must be > 0, got {}", size.0);
        assert_pyval!(size.1 > 0, "Height must be > 0, got {}", size.0);

        self.window.set_size((size.0 as u32, size.1 as u32));
        self.window.window.swap_buffers();
        Ok(())
    }

    /// Returns the frame position.
    pub fn get_position(&self) -> PyResult<(i32, i32)> {
        let pos: (i32, i32) = match self.window.get_position() {
            Some(position) => position.into(),
            None => (0, 0),
        };
        Ok(pos)
    }

    /// Sets the frame position.
    pub fn set_position(&mut self, pos: (i32, i32)) -> PyResult<()> {
        self.window.set_position(pos);
        Ok(())
    }
}

#[pyproto]
impl PyGCProtocol for Frame {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        if let Some(ref handler) = self.draw_handler {
            visit.call(handler)?
        }
        if let Some(ref handler) = self.event_handler {
            visit.call(handler)?
        }
        Ok(())
    }

    fn __clear__(&mut self) {
        if let Some(handler) = self.draw_handler.take() {
            self.py().release(handler);
        }
        if let Some(handler) = self.event_handler.take() {
            self.py().release(handler);
        }
    }
}

pub fn create_frame<'p>(py: &'p Python, title: String, width: u32, height: u32, resizable: bool, fullscreen: bool) -> Py<Frame> {
    const OPENGL_VERSION: OpenGL = OpenGL::V3_3;
    const SAMPLES: u8 = 4;

    let settings = WindowSettings::new(title, (width, height))
        .resizable(resizable)
        .fullscreen(fullscreen)
        .opengl(OPENGL_VERSION)
        .samples(SAMPLES)
        .vsync(true)
        .srgb(true);

    let window: FrameWindow = FrameWindow::new(
        OPENGL_VERSION,
        SAMPLES,
        settings.build().unwrap(),
    );

    py.init(|token| Frame {
        window,
        canvas: Canvas::new(py),
        draw_handler: None,
        event_handler: None,
        started: false,
        token,
    }).unwrap()
}