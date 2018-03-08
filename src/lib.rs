#![feature(proc_macro, specialization, const_fn)]

extern crate piston_window;
extern crate glfw_window;
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::py::modinit as pymodinit;

#[macro_use]
mod macros;

mod frame;
mod canvas;

#[pymodinit(pygui)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<frame::Frame>()?;
    m.add_class::<canvas::Canvas>()?;

    #[pyfn(m, "create_frame")]
    fn create_frame(py: Python, title: String, width: i32, height: i32, resizable: Option<bool>, fullscreen: Option<bool>) -> PyResult<Py<frame::Frame>> {
        let resizable = resizable.unwrap_or(true);
        let fullscreen = fullscreen.unwrap_or(false);

        assert_pyval!(width > 0, "Width must be > 0, got {}", width);
        assert_pyval!(height > 0, "Height must be > 0, got {}", width);

        Ok(frame::create_frame(&py, title, width as u32, height as u32, resizable, fullscreen))
    }

    Ok(())
}