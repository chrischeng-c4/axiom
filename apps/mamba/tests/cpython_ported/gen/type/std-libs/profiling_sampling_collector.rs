use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_collector/Collector__export__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_collector_Collector__export__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_collector"
# dimension = "type"
# case = "Collector__export__filename_as_StrOrBytesPath_wrong"
# subject = "profiling.sampling.collector.Collector.export(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.collector.Collector.export(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.collector import Collector
obj = object.__new__(Collector)
try:
    obj.export(_W())  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_collector/extract_lineno__location_as__Location_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_collector_extract_lineno__location_as__Location_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_collector"
# dimension = "type"
# case = "extract_lineno__location_as__Location_wrong"
# subject = "profiling.sampling.collector.extract_lineno(location: _Location)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.collector.extract_lineno(location: _Location); call it with the wrong type.

typeshed contract: location is _Location. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.collector import extract_lineno
try:
    extract_lineno(_W())  # location: _Location <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_collector/normalize_location__location_as__Location_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_collector_normalize_location__location_as__Location_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_collector"
# dimension = "type"
# case = "normalize_location__location_as__Location_wrong"
# subject = "profiling.sampling.collector.normalize_location(location: _Location)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.collector.normalize_location(location: _Location); call it with the wrong type.

typeshed contract: location is _Location. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.collector import normalize_location
try:
    normalize_location(_W())  # location: _Location <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
