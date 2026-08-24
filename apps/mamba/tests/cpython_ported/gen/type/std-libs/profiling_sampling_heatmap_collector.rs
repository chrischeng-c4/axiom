use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_heatmap_collector/HeatmapCollector__export__output_path_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_heatmap_collector_HeatmapCollector__export__output_path_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_heatmap_collector"
# dimension = "type"
# case = "HeatmapCollector__export__output_path_as_StrOrBytesPath_wrong"
# subject = "profiling.sampling.heatmap_collector.HeatmapCollector.export(output_path: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/heatmap_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.heatmap_collector.HeatmapCollector.export(output_path: StrOrBytesPath); call it with the wrong type.

typeshed contract: output_path is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.heatmap_collector import HeatmapCollector
obj = object.__new__(HeatmapCollector)
try:
    obj.export(_W())  # output_path: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_heatmap_collector/HeatmapCollector__init__sample_interval_usec_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_heatmap_collector_HeatmapCollector__init__sample_interval_usec_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_heatmap_collector"
# dimension = "type"
# case = "HeatmapCollector__init__sample_interval_usec_as_int_wrong"
# subject = "profiling.sampling.heatmap_collector.HeatmapCollector.__init__(sample_interval_usec: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/heatmap_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.heatmap_collector.HeatmapCollector.__init__(sample_interval_usec: int); call it with the wrong type.

typeshed contract: sample_interval_usec is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.heatmap_collector import HeatmapCollector
try:
    HeatmapCollector("not_an_int")  # sample_interval_usec: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_heatmap_collector/HeatmapCollector__set_stats__sample_interval_usec_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_heatmap_collector_HeatmapCollector__set_stats__sample_interval_usec_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_heatmap_collector"
# dimension = "type"
# case = "HeatmapCollector__set_stats__sample_interval_usec_as_int_wrong"
# subject = "profiling.sampling.heatmap_collector.HeatmapCollector.set_stats(sample_interval_usec: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/heatmap_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.heatmap_collector.HeatmapCollector.set_stats(sample_interval_usec: int); call it with the wrong type.

typeshed contract: sample_interval_usec is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.heatmap_collector import HeatmapCollector
obj = object.__new__(HeatmapCollector)
try:
    obj.set_stats("not_an_int", 0.0, 0.0)  # sample_interval_usec: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
