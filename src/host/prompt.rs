//! Show user popup window to ask for some value.
//!
//! Firstly, init a builder either with [`Prompt::builder`](struct.Prompt.html#method.builder) or
//! [`PromptBuilder::default`] to build your prompt, then use its
//! [`PromptBuilder::show`](struct.PromptBuilder.html#method.show) method to get the result
//! ([`Prompt`](struct.Prompt.html)).
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

use crate::host::Host;
use crate::AsRawPtr;

/// The result returned by [`PromptBuilder::show`](../struct.PromptBuilder.html#method.show) if the
/// user closed the window by clicking OK.
#[derive(Debug)]
pub struct Prompt {
    /// The text typed in by the user.
    pub value: String,
    /// The color selected by the user, if [`PromptBuilder::with_color`] called. Otherwise it's
    /// `None`.
    pub color: Option<i32>,
}

impl Prompt {
    /// Init [`PromptBuilder`](../struct.PromptBuilder.html).
    pub fn builder() -> PromptBuilder {
        Default::default()
    }
}

/// Use this to show [`Prompt`](../struct.Promtp.html).
#[derive(Debug, Default)]
pub struct PromptBuilder {
    x: Option<u32>,
    y: Option<u32>,
    with_color: bool,
}

impl PromptBuilder {
    /// Set horizontal position. Otherwise it's centered horizontally.
    pub fn with_x(mut self, x: u32) -> Self {
        self.x = Some(x);
        self
    }

    /// Set vertical position. Otherwise it's centered vertically.
    pub fn with_y(mut self, y: u32) -> Self {
        self.y = Some(y);
        self
    }

    /// Call if you want user to set color also.
    pub fn with_color(mut self) -> Self {
        self.with_color = true;
        self
    }

    /// Show prompt the user and return [`Prompt`](../struct.Prompt.html), if he/she closed the
    /// window by clicking OK. The method returns `None` otherwise.
    pub fn show(self, host: &mut Host, message: String) -> Option<Prompt> {
        let mut color = self.with_color as c_int - 1;
        let value = CString::default().into_raw();

        if unsafe {
            !prompt_show(
                *host.host_ptr.get_mut(),
                self.x.map(|v| v as c_int).unwrap_or(-1),
                self.y.map(|v| v as c_int).unwrap_or(-1),
                message.as_raw_ptr() as *mut c_char,
                value,
                &mut color,
            )
        } {
            unsafe { CString::from_raw(value) };
            return None;
        }

        Some(Prompt {
            value: self.value_from_raw(value),
            color: self.color_to_result(color),
        })
    }

    fn value_from_raw(&self, ptr: *mut c_char) -> String {
        unsafe { CString::from_raw(ptr) }
            .to_string_lossy()
            .to_string()
    }

    fn color_to_result(&self, color: c_int) -> Option<i32> {
        if self.with_color {
            Some(i32::from_be(color))
        } else {
            None
        }
    }
}

extern "C" {
    fn prompt_show(
        host: *mut c_void,
        x: c_int,
        y: c_int,
        msg: *mut c_char,
        result: *mut c_char,
        color: &mut c_int,
    ) -> bool;
}
