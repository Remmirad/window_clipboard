use crate::ClipboardProvider;

use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use std::error::Error;

#[cfg(feature = "wayland")]
pub use clipboard_wayland as wayland;
#[cfg(feature = "x11")]
pub use clipboard_x11 as x11;

#[derive(Debug)]
struct LinuxClipboardError;

impl std::fmt::Display for LinuxClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("This window server's clipboard feature is not enabled")
    }
}

impl Error for LinuxClipboardError {}

pub unsafe fn connect<W: HasDisplayHandle>(
    window: &W,
) -> Result<Box<dyn ClipboardProvider>, Box<dyn Error>> {
    let clipboard = match window.display_handle()?.as_raw() {
        #[cfg(feature = "wayland")]
        RawDisplayHandle::Wayland(handle) => {
            Box::new(wayland::Clipboard::connect(handle.display.as_ptr())) as _
        }
        #[cfg(feature = "x11")]
        RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_) => {
            Box::new(x11::Clipboard::connect()?) as _
        }
        _ => Err(LinuxClipboardError)?,
    };

    Ok(clipboard)
}

#[cfg(feature = "wayland")]
impl ClipboardProvider for wayland::Clipboard {
    fn read(&self) -> Result<String, Box<dyn Error>> {
        self.read()
    }

    fn read_content(&self) -> Result<crate::ClipboardContent, Box<dyn Error>> {
        let c = self.read_content()?;
        Ok(match c.data {
            wayland::ClipboardData::String(s) => {
                crate::ClipboardContent::String(s)
            }
            wayland::ClipboardData::Paths(path_bufs) => {
                crate::ClipboardContent::Paths(path_bufs)
            }
            wayland::ClipboardData::Cbor(data) => {
                crate::ClipboardContent::Cbor(data)
            }
            wayland::ClipboardData::Image(data)
                if c.mime == wayland::MimeType::ImageBmp =>
            {
                crate::ClipboardContent::BmpImage(data)
            }
            wayland::ClipboardData::Image(data)
                if c.mime == wayland::MimeType::ImagePng =>
            {
                crate::ClipboardContent::PngImage(data)
            }
            wayland::ClipboardData::Image(data)
                if c.mime == wayland::MimeType::ImageJpeg =>
            {
                crate::ClipboardContent::JpegImage(data)
            }
            _ => return Err("Unknown image format".into()),
        })
    }

    fn read_primary(&self) -> Option<Result<String, Box<dyn Error>>> {
        Some(self.read_primary())
    }

    fn write(&mut self, contents: String) -> Result<(), Box<dyn Error>> {
        self.write(contents)
    }

    fn write_content(
        &mut self,
        contents: crate::ClipboardContent,
    ) -> Result<(), Box<dyn Error>> {
        let content = match contents {
            crate::ClipboardContent::String(s) => wayland::ClipboardContent {
                mime: wayland::MimeType::Utf8String,
                data: wayland::ClipboardData::String(s),
            },
            crate::ClipboardContent::Paths(path_bufs) => {
                wayland::ClipboardContent {
                    mime: wayland::MimeType::TextUriList,
                    data: wayland::ClipboardData::Paths(path_bufs),
                }
            }
            crate::ClipboardContent::Cbor(data) => wayland::ClipboardContent {
                mime: wayland::MimeType::ApplicationCbor,
                data: wayland::ClipboardData::Cbor(data),
            },
            crate::ClipboardContent::BmpImage(img) => {
                wayland::ClipboardContent {
                    mime: wayland::MimeType::ImageBmp,
                    data: wayland::ClipboardData::Image(img),
                }
            }
            crate::ClipboardContent::PngImage(img) => {
                wayland::ClipboardContent {
                    mime: wayland::MimeType::ImagePng,
                    data: wayland::ClipboardData::Image(img),
                }
            }
            crate::ClipboardContent::JpegImage(img) => {
                wayland::ClipboardContent {
                    mime: wayland::MimeType::ImageJpeg,
                    data: wayland::ClipboardData::Image(img),
                }
            }
        };
        self.write_content(content)
    }

    fn write_primary(
        &mut self,
        contents: String,
    ) -> Option<Result<(), Box<dyn Error>>> {
        Some(self.write_primary(contents))
    }
}

#[cfg(feature = "x11")]
impl ClipboardProvider for x11::Clipboard {
    fn read(&self) -> Result<String, Box<dyn Error>> {
        self.read().map_err(Box::from)
    }

    fn read_primary(&self) -> Option<Result<String, Box<dyn Error>>> {
        Some(self.read_primary().map_err(Box::from))
    }

    fn write(&mut self, contents: String) -> Result<(), Box<dyn Error>> {
        self.write(contents).map_err(Box::from)
    }

    fn write_primary(
        &mut self,
        contents: String,
    ) -> Option<Result<(), Box<dyn Error>>> {
        Some(self.write_primary(contents).map_err(Box::from))
    }
}
