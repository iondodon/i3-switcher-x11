use std::ffi::CString;

use gdk4::gdk_pixbuf::{Colorspace, Pixbuf};
use xcap::image::{ImageBuffer, Rgba};
use xcap::Monitor;

pub fn take(monitor_name: &CString) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        if monitor.name().eq(monitor_name.to_str().unwrap()) {
            let image = monitor.capture_image();
            return match image {
                Ok(image) => Some(image),
                Err(err) => {
                    log::error!("Could not take screenshot: {}", err);
                    None
                }
            };
        }
    }

    None
}

pub fn rgba_image_to_pixbuf(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Pixbuf {
    let width = img.width() as i32;
    let height = img.height() as i32;
    let row_stride = img.sample_layout().height_stride as i32;

    Pixbuf::from_mut_slice(
        img.clone().into_raw(),
        Colorspace::Rgb,
        true,
        8,
        width,
        height,
        row_stride,
    )
}
