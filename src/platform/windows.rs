use crate::ClipboardProvider;

use clipboard_win::{formats::RawData, get_clipboard_string, set_clipboard_string};
use raw_window_handle::HasDisplayHandle;

use std::{borrow::Cow, error::Error, path::PathBuf};

pub fn connect<W: HasDisplayHandle>(
    _window: &W,
) -> Result<Box<dyn ClipboardProvider>, Box<dyn Error>> {
    Ok(Box::new(Clipboard))
}

pub struct Clipboard;

impl ClipboardProvider for Clipboard {
    fn read(&self) -> Result<String, Box<dyn Error>> {
        //clipboard_win::
        Ok(get_clipboard_string()?)
    }

    fn read_content(&self) -> Result<crate::ClipboardContent, Box<dyn Error>> {
        if clipboard_win::is_format_avail(clipboard_win::formats::CF_BITMAP) {
            Ok(crate::ClipboardContent::BmpImage(clipboard_win::get_clipboard(clipboard_win::formats::Bitmap)?))
        }else if clipboard_win::is_format_avail(clipboard_win::formats::CF_HDROP) {
            let paths = clipboard_win::get_clipboard::<Vec<PathBuf>,_>(clipboard_win::formats::FileList)?;
            Ok(crate::ClipboardContent::Paths(paths))
        } else if clipboard_win::is_format_avail(clipboard_win::formats::CF_UNICODETEXT) {
            Ok(crate::ClipboardContent::String(get_clipboard_string()?))
        } else if clipboard_win::is_format_avail(clipboard_win::formats::CF_PRIVATEFIRST+0xF0) {
            Ok(crate::ClipboardContent::Cbor(clipboard_win::get_clipboard(RawData(clipboard_win::formats::CF_PRIVATEFIRST+0xF0))?))
        }else {
            Err("Empty".into())
        }
    }

    fn write(&mut self, contents: String) -> Result<(), Box<dyn Error>> {
        Ok(set_clipboard_string(&contents)?)
    }

    fn write_content(
            &mut self,
            contents: crate::ClipboardContent,
        ) -> Result<(), Box<dyn Error>> {
        match contents {
            crate::ClipboardContent::String(s) =>  set_clipboard_string(&s)?,
            crate::ClipboardContent::Paths(path_bufs) => {
                let paths: Vec<Cow<'_,str>> = path_bufs.iter().map(|p|p.to_string_lossy()).collect();
                clipboard_win::raw::set_file_list(paths.as_slice())?;
            },
            crate::ClipboardContent::Cbor(data) => {
                clipboard_win::set_clipboard(RawData(clipboard_win::formats::CF_PRIVATEFIRST+0xF0), data)?;
            },
            crate::ClipboardContent::BmpImage(_) => todo!(),
            crate::ClipboardContent::PngImage(_) => todo!(),
            crate::ClipboardContent::JpegImage(_) => todo!(),
        };
        Ok(())
    }
}
