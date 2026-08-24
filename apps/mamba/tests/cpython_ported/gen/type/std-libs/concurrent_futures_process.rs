use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_process/ProcessPoolExecutor__init__max_workers_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_process_ProcessPoolExecutor__init__max_workers_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_process"
# dimension = "type"
# case = "ProcessPoolExecutor__init__max_workers_as_typed_wrong"
# subject = "concurrent.futures.process.ProcessPoolExecutor.__init__(max_workers: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/process.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.process.ProcessPoolExecutor.__init__(max_workers: typed); call it with the wrong type.

typeshed contract: max_workers is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.process import ProcessPoolExecutor
try:
    ProcessPoolExecutor(_W())  # max_workers: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
