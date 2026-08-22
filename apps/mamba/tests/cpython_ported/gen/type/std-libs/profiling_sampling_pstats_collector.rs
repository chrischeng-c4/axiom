use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_pstats_collector/PstatsCollector__export__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_pstats_collector_PstatsCollector__export__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_pstats_collector"
# dimension = "type"
# case = "PstatsCollector__export__filename_as_StrOrBytesPath_wrong"
# subject = "profiling.sampling.pstats_collector.PstatsCollector.export(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/pstats_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.pstats_collector.PstatsCollector.export(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.pstats_collector import PstatsCollector
obj = object.__new__(PstatsCollector)
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

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_pstats_collector/PstatsCollector__init__sample_interval_usec_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_pstats_collector_PstatsCollector__init__sample_interval_usec_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_pstats_collector"
# dimension = "type"
# case = "PstatsCollector__init__sample_interval_usec_as_int_wrong"
# subject = "profiling.sampling.pstats_collector.PstatsCollector.__init__(sample_interval_usec: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/pstats_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.pstats_collector.PstatsCollector.__init__(sample_interval_usec: int); call it with the wrong type.

typeshed contract: sample_interval_usec is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.pstats_collector import PstatsCollector
try:
    PstatsCollector("not_an_int")  # sample_interval_usec: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_pstats_collector/PstatsCollector__print_stats__sort_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_pstats_collector_PstatsCollector__print_stats__sort_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_pstats_collector"
# dimension = "type"
# case = "PstatsCollector__print_stats__sort_as_int_wrong"
# subject = "profiling.sampling.pstats_collector.PstatsCollector.print_stats(sort: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/pstats_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.pstats_collector.PstatsCollector.print_stats(sort: int); call it with the wrong type.

typeshed contract: sort is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.pstats_collector import PstatsCollector
obj = object.__new__(PstatsCollector)
try:
    obj.print_stats("not_an_int")  # sort: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
