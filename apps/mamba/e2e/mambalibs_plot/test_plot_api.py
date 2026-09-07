"""The Python-visible contract of ``mambalibs.plot``, over ``plotkit``'s ``viz``.

Fixture material, not a case: ``mambalibs_plot_wheel.rs`` runs this directory
with ``python -m pytest -p no:cacheprovider`` inside the environment it built.
``test_plot_namespace.py`` owns where the module lives; this file owns what it
does.

Every assertion below maps onto a function, type, or literal that already
exists in ``apps/mamba/mambalibs/plotkit/src/viz``. The work item freezes
"existing core API only, no matplotlib-parity promise and no rendering backend
beyond what ``viz`` already does", so nothing here asks for behaviour the Rust
crate does not implement, and each expected value is derived from the Rust
source rather than from Matplotlib.

Shape of the surface
--------------------

``plotkit`` is one crate of one module (``src/lib.rs`` has only ``pub mod
viz;``), and ``viz/mod.rs`` re-exports its public names at the module root. The
binding keeps that shape flat, exactly as ``mambalibs.array`` does for
``arraykit``'s single ``array`` module: there is no ``plot.viz`` submodule,
because a level that always has one member names nothing.

Naming rules
------------

The wheel is a binding, so the Python names follow the Rust ones by rule, the
same rules ``mambalibs.array`` and ``mambalibs.sci`` already ship:

* a Rust free function becomes a module-level function of the same snake_case
  name, taking the same arguments in the same positional order;
* a ``pub struct`` or ``pub enum`` becomes a class of the same PascalCase name,
  and a fieldless enum publishes each variant as a class attribute
  (``ThemeName.Dark``), exactly as ``DType.Float32`` does;
* a public field becomes a Python attribute (``TickInfo.step``);
* an associated constructor such as ``DataSeries::line`` becomes a classmethod
  of the same name; ``Chart::new`` becomes ``__init__``;
* ``Result<T, E>`` raises a Python exception -- every ``VizError`` variant is a
  ``ValueError``, because each one is an argument the caller could have chosen
  differently;
* ``Vec<f64>`` is a ``list[float]``, ``Vec<String>`` a ``list[str]``, and
  ``String`` a ``str``.

The names this contract requires
--------------------------------

* ``plot.Chart``: ``Chart()``, and the fluent methods ``.title(str)``,
  ``.size(width, height)``, ``.x_label(str)``, ``.y_label(str)``,
  ``.theme(ThemeName)``, ``.add_series(DataSeries)``, each returning a
  ``Chart`` so calls chain, plus ``.to_svg() -> str``.
* ``plot.DataSeries``: the classmethods ``.line(x, y)``, ``.bar(labels,
  values)``, ``.scatter(x, y)``.
* ``plot.ThemeName``: the four variants ``Light``, ``Dark``, ``Minimal``,
  ``Publication``.
* ``plot.to_html(chart) -> str``.
* ``plot.nice_ticks(data_min, data_max, tick_count) -> TickInfo``, and
  ``plot.TickInfo`` carrying ``.min``, ``.max``, ``.ticks``, ``.step``.
* ``plot.format_tick(value) -> str``.

The fluent methods are asserted only through chained calls, so an
implementation may return a fresh ``Chart`` or mutate and return ``self``: the
work item freezes neither, and pinning identity here would refuse a correct
binding for a reason ``viz`` does not have.

Nothing here writes a file. ``Chart::save_svg`` and ``export_chart`` exist in
``viz`` and are deliberately not part of this contract: a case that produces
output would then have to clean it up, and the strings are the whole of what
the rendering claim is about.
"""

import pytest

import mambalibs.plot as plot

#: A four-point line, small enough that the tick arithmetic below stays exact.
LINE_X = [1.0, 2.0, 3.0, 4.0]
LINE_Y = [10.0, 20.0, 15.0, 25.0]


def _line_chart(title=None):
    chart = plot.Chart()
    if title is not None:
        chart = chart.title(title)
    return chart.add_series(plot.DataSeries.line(LINE_X, LINE_Y))


# --- Chart, and the SVG it renders -----------------------------------------


def test_line_chart_renders_an_svg_document():
    """``Chart::to_svg`` builds a whole document, not a fragment.

    ``render/svg.rs`` opens with ``<svg xmlns=...>`` and closes with
    ``</svg>``; a line series is drawn as one ``<path>``.
    """
    svg = _line_chart("Quarterly revenue").size(400.0, 300.0).to_svg()

    assert svg.startswith("<svg"), svg[:80]
    assert svg.rstrip().endswith("</svg>"), svg[-80:]
    assert "<path" in svg
    assert "Quarterly revenue" in svg


def test_size_reaches_the_svg_viewport():
    """``Chart::size`` is the renderer's canvas, not a hint."""
    svg = _line_chart().size(400.0, 300.0).to_svg()

    assert 'width="400"' in svg, svg[:200]
    assert 'height="300"' in svg, svg[:200]


def test_bar_series_names_every_category():
    """``render_bar`` writes each label as its own ``<text>`` element."""
    svg = (
        plot.Chart()
        .add_series(plot.DataSeries.bar(["A", "B", "C"], [10.0, 20.0, 30.0]))
        .to_svg()
    )

    assert "<rect" in svg
    for label in ("A", "B", "C"):
        assert ">{}</text>".format(label) in svg, label


def test_scatter_series_draws_one_point_per_pair():
    """``render_scatter`` emits a ``<circle>`` per point and nothing else does."""
    svg = (
        plot.Chart()
        .add_series(plot.DataSeries.scatter([0.0, 1.0, 2.0], [0.0, 1.0, 4.0]))
        .to_svg()
    )

    assert svg.count("<circle") == 3, svg


def test_axis_labels_reach_the_svg():
    """``x_label``/``y_label`` route ``to_svg`` through ``render_full``."""
    svg = _line_chart().x_label("time (s)").y_label("amplitude").to_svg()

    assert "time (s)" in svg
    assert "amplitude" in svg


def test_a_chart_with_no_series_refuses_to_render():
    """``VizError::EmptyData`` is a ``ValueError``, and it says why."""
    with pytest.raises(ValueError) as excinfo:
        plot.Chart().to_svg()

    assert "no data series added" in str(excinfo.value)


# --- themes -----------------------------------------------------------------


def test_theme_name_publishes_the_four_presets():
    """``ThemeName`` is fieldless, so each variant is a class attribute."""
    for variant in ("Light", "Dark", "Minimal", "Publication"):
        assert hasattr(plot.ThemeName, variant), variant


def test_dark_theme_repaints_the_chart():
    """``Theme::dark`` is ``#1e1e2e`` on ``#cdd6f4``; ``Light`` is neither."""
    dark = _line_chart("Themed").theme(plot.ThemeName.Dark).to_svg()
    light = _line_chart("Themed").theme(plot.ThemeName.Light).to_svg()

    assert "#1e1e2e" in dark
    assert "#cdd6f4" in dark
    assert "#1e1e2e" not in light
    assert "#cdd6f4" not in light


# --- export -----------------------------------------------------------------


def test_to_html_wraps_the_chart_svg():
    """``export::to_html`` embeds ``to_svg()`` verbatim under the chart title."""
    chart = _line_chart("Embedded")
    html = plot.to_html(chart)

    assert html.startswith("<!DOCTYPE html>"), html[:80]
    assert "<title>Embedded</title>" in html
    assert chart.to_svg() in html


# --- axis -------------------------------------------------------------------


def test_nice_ticks_rounds_a_range_to_a_nice_step():
    """``nice_ticks(0, 100, 5)`` is the Wilkinson step 20 over six ticks."""
    info = plot.nice_ticks(0.0, 100.0, 5)

    assert info.min == pytest.approx(0.0)
    assert info.max == pytest.approx(100.0)
    assert info.step == pytest.approx(20.0)
    assert len(info.ticks) == 6, info.ticks
    assert info.ticks == pytest.approx([0.0, 20.0, 40.0, 60.0, 80.0, 100.0])


def test_nice_ticks_widens_a_constant_range():
    """Constant data would divide by zero, so ``viz`` opens it to +/-1."""
    info = plot.nice_ticks(5.0, 5.0, 5)

    assert info.min == pytest.approx(4.0)
    assert info.max == pytest.approx(6.0)
    assert info.ticks == pytest.approx([4.0, 5.0, 6.0])


def test_format_tick_picks_a_notation_per_magnitude():
    """The four branches of ``format_tick``, one value each."""
    assert plot.format_tick(0.0) == "0"
    assert plot.format_tick(20.0) == "20"
    assert plot.format_tick(0.5) == "0.5"
    assert plot.format_tick(1500000.0) == "1.5e6"
