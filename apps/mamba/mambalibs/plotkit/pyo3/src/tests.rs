//! Colocated tests for [`crate::core`], the interpreter-free half of the
//! binding.
//!
//! These run under `cargo test -p plotkit-pyo3 --lib` with no Python present:
//! the module's own name, the theme registry, the refusal messages, and the
//! argument checks that keep a chart with unplottable arguments from reaching
//! `viz` are all plain Rust, so they are judged here rather than only through
//! the wheel the e2e case builds.
//!
//! Every name below is prefixed `plot_`. `arraykit-pyo3` and `scikit-pyo3`
//! publish their own `tests::` modules in the same workspace, and the impl
//! phase keys a test result by its bare `module::function` path, so a name
//! shared with either crate would be reported twice under one key.

use plotkit::viz::{Chart, ThemeName, VizError};

use crate::core::{
    checked_bar, checked_line, checked_nice_ticks, checked_scatter, checked_size, render_html,
    render_svg, theme_from_name, PlotError, MAX_TICK_COUNT, MODULE_NAME, THEME_NAMES,
};

#[test]
fn plot_names_the_module_the_wheel_installs() {
    assert_eq!(MODULE_NAME, "mambalibs.plot");
}

#[test]
fn plot_publishes_the_four_theme_presets() {
    assert_eq!(
        THEME_NAMES.to_vec(),
        vec!["Light", "Dark", "Minimal", "Publication"]
    );
    assert_eq!(theme_from_name("Light"), Some(ThemeName::Light));
    assert_eq!(theme_from_name("Dark"), Some(ThemeName::Dark));
    assert_eq!(theme_from_name("Minimal"), Some(ThemeName::Minimal));
    assert_eq!(theme_from_name("Publication"), Some(ThemeName::Publication));
}

#[test]
fn plot_an_unknown_theme_name_selects_no_preset() {
    assert_eq!(theme_from_name("Neon"), None);
    assert_eq!(theme_from_name("dark"), None);
}

#[test]
fn plot_a_refusal_keeps_the_reason_viz_gave() {
    let err = PlotError::from(VizError::EmptyData("no data series added".into()));
    assert!(
        err.message().contains("no data series added"),
        "{}",
        err.message()
    );
}

#[test]
fn plot_a_line_series_needs_one_y_per_x() {
    let err = checked_line(vec![1.0, 2.0, 3.0], vec![10.0, 20.0])
        .expect_err("3 x values against 2 y values is not a line");
    assert!(err.message().contains('3') && err.message().contains('2'), "{}", err.message());
}

#[test]
fn plot_a_line_series_refuses_a_value_that_cannot_be_plotted() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let err = checked_line(vec![1.0, 2.0], vec![10.0, bad])
            .err()
            .unwrap_or_else(|| panic!("{bad} is not a coordinate and must be refused"));
        assert!(err.message().contains("finite"), "{}", err.message());
        assert!(checked_line(vec![1.0, bad], vec![10.0, 20.0]).is_err());
    }
}

#[test]
fn plot_a_line_series_refuses_empty_data() {
    let err = checked_line(Vec::new(), Vec::new()).expect_err("an empty series plots nothing");
    assert!(err.message().contains("empty"), "{}", err.message());
}

#[test]
fn plot_a_scatter_series_needs_one_y_per_x() {
    assert!(checked_scatter(vec![1.0, 2.0], vec![1.0]).is_err());
    assert!(checked_scatter(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 4.0]).is_ok());
}

#[test]
fn plot_a_bar_series_needs_one_value_per_label() {
    let err = checked_bar(vec!["A".into(), "B".into()], vec![10.0])
        .expect_err("2 labels against 1 value is not a bar chart");
    assert!(err.message().contains("label"), "{}", err.message());
    assert!(checked_bar(
        vec!["A".into(), "B".into(), "C".into()],
        vec![10.0, 20.0, 30.0]
    )
    .is_ok());
}

#[test]
fn plot_a_canvas_must_have_a_positive_finite_size() {
    assert_eq!(checked_size(400.0, 300.0), Ok((400.0, 300.0)));
    assert!(checked_size(0.0, 300.0).is_err());
    assert!(checked_size(400.0, -1.0).is_err());
    assert!(checked_size(f64::NAN, 300.0).is_err());
}

#[test]
fn plot_nice_ticks_rounds_a_range_the_way_viz_does() {
    let info = checked_nice_ticks(0.0, 100.0, 5).expect("0..100 over 5 ticks is a plottable range");
    assert_eq!(info.step, 20.0);
    assert_eq!(info.ticks, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
}

#[test]
fn plot_nice_ticks_bounds_what_it_will_lay_out() {
    assert!(checked_nice_ticks(0.0, 1.0, 0).is_err());
    assert!(checked_nice_ticks(0.0, 1.0, MAX_TICK_COUNT + 1).is_err());
    assert!(checked_nice_ticks(0.0, 1.0, MAX_TICK_COUNT).is_ok());
    assert!(checked_nice_ticks(f64::NAN, 1.0, 5).is_err());
    assert!(checked_nice_ticks(0.0, f64::INFINITY, 5).is_err());
}

#[test]
fn plot_a_chart_with_no_series_refuses_to_render() {
    let err = render_svg(&Chart::new()).expect_err("an empty chart is not a document");
    assert!(
        err.message().contains("no data series added"),
        "{}",
        err.message()
    );
}

#[test]
fn plot_a_line_chart_renders_one_path_into_a_whole_document() {
    let chart = Chart::new()
        .title("Quarterly revenue")
        .size(400.0, 300.0)
        .add_series(checked_line(vec![1.0, 2.0, 3.0, 4.0], vec![10.0, 20.0, 15.0, 25.0]).unwrap());
    let svg = render_svg(&chart).expect("a line chart renders");
    assert!(svg.starts_with("<svg"), "{}", &svg[..80.min(svg.len())]);
    assert!(svg.trim_end().ends_with("</svg>"));
    assert!(svg.contains("<path"));
    assert!(svg.contains("Quarterly revenue"));
}

#[test]
fn plot_html_embeds_the_chart_svg_verbatim() {
    let chart = Chart::new()
        .title("Embedded")
        .add_series(checked_line(vec![1.0, 2.0], vec![10.0, 20.0]).unwrap());
    let html = render_html(&chart).expect("a rendering chart exports");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Embedded</title>"));
    assert!(html.contains(&render_svg(&chart).unwrap()));
}
