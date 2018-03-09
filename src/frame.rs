use pyo3::prelude::*;

use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;
use pyo3::py::proto as pyproto;

use piston_window::*;
use glfw_window::GlfwWindow;
use glfw;

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

                    let ret = handler.call(py, args, NoArgs);
                    if let Err(err) = ret {
                        return Err(err);
                    }
                }

                self.window.draw_2d(&e, |c, g| {
                    // Draw the canvas.
                    let py = gil.python();
                    let canvas = canvas.as_mut(py);
                    canvas.draw_canvas(&c, g)
                });
            }

            // TODO: event handling
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

    // Only supported with GLFW.

    /// Returns `true` if the frame is focused.
    pub fn is_focused(&self) -> PyResult<()> {
        let w = self.get_glfw_window();
        w.is_focused();
        Ok(())
    }

    /// Focuses the frame.
    pub fn focus(&mut self) -> PyResult<()> {
        let w = self.get_glfw_window_mut();
        w.focus();
        Ok(())
    }

    /// Returns `true` if the frame is maximized.
    pub fn is_maximized(&self) -> PyResult<bool> {
        let w = self.get_glfw_window();
        Ok(w.is_maximized())
    }

    /// Maximizes the frame.
    pub fn maximize(&mut self) -> PyResult<()> {
        let w = self.get_glfw_window_mut();
        w.maximize();
        Ok(())
    }

    /// Returns `true` if the frame is minimized.
    pub fn is_minimized(&self) -> PyResult<bool> {
        let w = self.get_glfw_window();
        Ok(w.is_iconified())
    }

    /// Minimizes the frame.
    pub fn minimize(&mut self) -> PyResult<()> {
        let w = self.get_glfw_window_mut();
        w.iconify();
        Ok(())
    }

    /// Restores the frame to a normal state.
    pub fn restore(&mut self) -> PyResult<()> {
        let w = self.get_glfw_window_mut();
        w.restore();
        Ok(())
    }

    /// Returns `true` if the frame is fullscreen.
    pub fn is_fullscreen(&self) -> PyResult<bool> {
        let mut fullscreen = false;
        let w = self.get_glfw_window();
        w.with_window_mode_mut(|m| {
            match m {
                glfw::WindowMode::FullScreen(..) => fullscreen = true,
                _ => (),
            }
        });

        Ok(fullscreen)
    }

    // DEBUG: not sure if we have a way of escaping fullscreen.
    /// Set the frame to fullscreen.
    pub fn set_fullscreen(&mut self, resolution: Option<(i32, i32)>) -> PyResult<()> {
        if let Some(res) = resolution {
            assert_pyval!(res.0 > 0, "Resolution width must be > 0, got {}", res.0);
            assert_pyval!(res.1 > 0, "Resolution height must be > 0, got {}", res.1);
        }

        let w = self.get_glfw_window_mut();
        let mut glfw = w.glfw;
        glfw.with_primary_monitor_mut(|_, m| {
            let monitor = m.unwrap();

            let mode = monitor.get_video_mode().unwrap();
            let res: (u32, u32) = if let Some(res) = resolution {
                (res.0 as u32, res.0 as u32)
            } else {
                (mode.width, mode.height)
            };

            w.set_monitor(
                glfw::WindowMode::FullScreen(&monitor),
                0, 0,
                res.0, res.1,
                Some(mode.refresh_rate),
            );
        });
        Ok(())
    }
}

impl Frame {
    fn get_glfw_window(&self) -> &glfw::Window {
        let ref w: FrameWindow = self.window;
        let ref w2: GlfwWindow = w.window;
        &w2.window
    }

    fn get_glfw_window_mut(&mut self) -> &mut glfw::Window {
        let ref mut w: FrameWindow = self.window;
        let ref mut w2: GlfwWindow = w.window;
        &mut w2.window
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
        .fullscreen(fullscreen) // DEBUG: fullscreen appears to be broken
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