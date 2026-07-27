use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_jsonl_collector/JsonlCollector__export__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_jsonl_collector_JsonlCollector__export__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_jsonl_collector"
# dimension = "type"
# case = "JsonlCollector__export__filename_as_StrOrBytesPath_wrong"
# subject = "profiling.sampling.jsonl_collector.JsonlCollector.export(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/jsonl_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.jsonl_collector.JsonlCollector.export(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from profiling.sampling.jsonl_collector import JsonlCollector
obj = object.__new__(JsonlCollector)
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

/// Ported from `tests/cpython/type/std-libs/profiling_sampling_jsonl_collector/JsonlCollector__init__sample_interval_usec_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_profiling_sampling_jsonl_collector_JsonlCollector__init__sample_interval_usec_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profiling_sampling_jsonl_collector"
# dimension = "type"
# case = "JsonlCollector__init__sample_interval_usec_as_int_wrong"
# subject = "profiling.sampling.jsonl_collector.JsonlCollector.__init__(sample_interval_usec: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profiling/sampling/jsonl_collector.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profiling.sampling.jsonl_collector.JsonlCollector.__init__(sample_interval_usec: int); call it with the wrong type.

typeshed contract: sample_interval_usec is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profiling.sampling.jsonl_collector import JsonlCollector
try:
    JsonlCollector("not_an_int")  # sample_interval_usec: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
