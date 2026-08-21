//! Image processing module — pure Rust image manipulation.
//!
//! - **types**: Image struct, pixel formats
//! - **ops**: Resize, crop, rotate, flip, color space conversion
//! - **filter**: Convolution filters (blur, sharpen, edge detection, Canny)
//! - **transform**: Geometric transforms, histogram equalization
//! - **morphology**: Dilate, erode, median blur
//! - **io**: Image file I/O (imread, imwrite)

mod blend;
mod color;
mod edge;
mod filter;
mod histogram;
pub mod io;
mod morphology;
mod ops;
mod transform;
mod types;

pub use blend::{alpha_blend, blend, composite, gradient_h, BlendMode};
pub use color::{gray_to_rgb, hsv_to_rgb, lab_to_rgb, rgb_to_gray, rgb_to_hsv, rgb_to_lab};
pub use edge::sobel;
pub use filter::{canny, gaussian_blur, sharpen, sobel_edges};
pub use histogram::{clahe, compute_histogram, equalize};
pub use io::{imdecode, imencode, imread, imwrite, ImageError};
pub use morphology::{dilate, erode, median_blur};
pub use ops::{
    crop, flip_horizontal, flip_vertical, resize, resize_bicubic, resize_bilinear,
    rgb_to_grayscale, rotate90,
};
pub use transform::{adjust_brightness, adjust_contrast, histogram_equalize, threshold};
pub use types::{Image, PixelFormat};
