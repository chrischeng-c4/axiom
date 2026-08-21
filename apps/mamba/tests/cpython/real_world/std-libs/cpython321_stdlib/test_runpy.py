# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_runpy"
# subject = "cpython321.test_runpy"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_runpy.py"
# status = "filled"
# ///
"""cpython321.test_runpy: execute CPython 3.12 seed test_runpy"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_runpy.py — #3449 axis-1 stdlib runpy AssertionPass seed.
#
# Mamba-authored seed exercising the `runpy` module surface called out
# in the issue:
#   run_module 'json.tool' style, run_path executes file, returns namespace.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. run_path on a written-to-tempfile script — returned namespace
#      carries top-level bindings (vars + functions).
#   3. run_path returns a dict; checks namespace metadata
#      (__name__ defaults to '<run_path>', __file__ equals the path).
#   4. run_path with run_name override.
#   5. run_module on a stdlib module that exposes safe top-level
#      bindings (`base64` — assert module-level functions are present
#      in the returned namespace).
#
# Boxed-int dodge (subtraction-against-zero) applied for length checks.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: runpy N asserts` to stdout.

import runpy
import os
import tempfile

_ledger: list[int] = []

# 1. Module identity + public surface.
assert runpy.__name__ == "runpy", "runpy.__name__"
_ledger.append(1)
assert hasattr(runpy, "run_module"), "exposes run_module"
_ledger.append(1)
assert hasattr(runpy, "run_path"), "exposes run_path"
_ledger.append(1)

# 2. run_path on a written-to-tempfile script.
_script = (
    "answer = 42\n"
    "name = 'mamba'\n"
    "def double(x):\n"
    "    return x * 2\n"
    "computed = double(answer)\n"
)
_tmpdir = tempfile.mkdtemp()
_path = os.path.join(_tmpdir, "seed_script.py")
_fh = open(_path, "w")
_fh.write(_script)
_fh.close()

_ns = runpy.run_path(_path)
assert isinstance(_ns, dict), "run_path returns a dict namespace"
_ledger.append(1)
assert _ns["answer"] == 42, "run_path namespace carries 'answer' = 42"
_ledger.append(1)
assert _ns["name"] == "mamba", "run_path namespace carries 'name' = 'mamba'"
_ledger.append(1)
assert _ns["computed"] == 84, "run_path executes top-level code (computed = double(42) = 84)"
_ledger.append(1)
assert callable(_ns["double"]) == True, "run_path namespace carries the def"
_ledger.append(1)
# Module metadata in the returned namespace.
assert "__name__" in _ns, "run_path namespace exposes __name__"
_ledger.append(1)
assert _ns["__name__"] == "<run_path>", "default __name__ == '<run_path>'"
_ledger.append(1)
assert "__file__" in _ns, "run_path namespace exposes __file__"
_ledger.append(1)
assert _ns["__file__"] == _path, "namespace __file__ matches the executed path"
_ledger.append(1)

# 3. run_path with explicit run_name override.
_ns2 = runpy.run_path(_path, run_name="my_custom_main")
assert _ns2["__name__"] == "my_custom_main", "run_name kwarg overrides __name__"
_ledger.append(1)
# Re-execution preserves the top-level bindings.
assert _ns2["answer"] == 42, "second run_path execution still binds 'answer'"
_ledger.append(1)
assert _ns2["computed"] == 84, "second run_path execution recomputes 'computed'"
_ledger.append(1)

# 4. run_path with init_globals seeded — pre-populated names visible.
_ns3 = runpy.run_path(_path, init_globals={"preset": 7})
assert _ns3["preset"] == 7, "init_globals seeded into namespace"
_ledger.append(1)
assert _ns3["answer"] == 42, "post-init_globals top-level code still runs"
_ledger.append(1)

# 5. run_module — execute a stdlib module that is safe to import as
# itself (no __main__ side-effects on `base64` when run_name != '__main__').
_ns_mod = runpy.run_module("base64", run_name="not_main_so_no_cli")
assert isinstance(_ns_mod, dict), "run_module returns a dict namespace"
_ledger.append(1)
# base64 exposes a set of canonical functions at module top level.
assert "b64encode" in _ns_mod, "run_module(base64) namespace carries b64encode"
_ledger.append(1)
assert "b64decode" in _ns_mod, "run_module(base64) namespace carries b64decode"
_ledger.append(1)
assert callable(_ns_mod["b64encode"]) == True, "namespace b64encode is callable"
_ledger.append(1)
assert _ns_mod["__name__"] == "not_main_so_no_cli", "run_module honours run_name kwarg"
_ledger.append(1)

# Cleanup the tempdir; not load-bearing for the assertion count.
os.remove(_path)
os.rmdir(_tmpdir)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: runpy {len(_ledger)} asserts")
