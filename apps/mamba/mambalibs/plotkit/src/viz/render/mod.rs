//! Rendering module for chart output.

pub mod svg;
mod svg_annotation;
mod svg_extra;
mod svg_legend;
mod svg_series;

pub use svg::SvgRenderer;
