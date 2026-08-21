# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_inspect"
# subject = "cpython321.test_inspect"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_inspect.py"
# status = "filled"
# ///
"""cpython321.test_inspect: execute CPython 3.12 seed test_inspect"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_inspect.py — #3437 axis-1 stdlib inspect AssertionPass seed.
#
# Mamba-authored seed exercising the `inspect` module surface called out
# in the issue:
#   signature parameters, getmembers, isclass/isfunction,
#   getsourcelines, getfullargspec.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. inspect.isclass / isfunction on real targets.
#   3. inspect.signature() — parameter names, defaults, kinds.
#   4. inspect.getfullargspec() — args / defaults / varargs / varkw.
#   5. inspect.getmembers() — predicate-filtered listing.
#   6. inspect.getsourcelines() — non-empty list + integer line number.
#   7. inspect.Parameter sentinels — empty / *args / **kwargs kinds.
#
# Boxed-int dodge (subtraction-against-zero) applied for equality on
# accumulated counts per the boxed-accumulator equality bug.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: inspect N asserts` to stdout.

import inspect

_ledger: list[int] = []


# Module-level def targets (top-level only — no nested closures).
def _target_no_defaults(a, b):
    return a + b


def _target_with_defaults(x, y=10, z="hi"):
    return (x, y, z)


def _target_varargs(*args, **kwargs):
    return (args, kwargs)


class _TargetClass:
    attr = 42

    def method(self, value):
        return value


# 1. Module identity + public surface.
assert inspect.__name__ == "inspect", "inspect.__name__"
_ledger.append(1)
assert hasattr(inspect, "signature"), "exposes signature"
_ledger.append(1)
assert hasattr(inspect, "getmembers"), "exposes getmembers"
_ledger.append(1)
assert hasattr(inspect, "isclass"), "exposes isclass"
_ledger.append(1)
assert hasattr(inspect, "isfunction"), "exposes isfunction"
_ledger.append(1)
assert hasattr(inspect, "getsourcelines"), "exposes getsourcelines"
_ledger.append(1)
assert hasattr(inspect, "getfullargspec"), "exposes getfullargspec"
_ledger.append(1)
assert hasattr(inspect, "Parameter"), "exposes Parameter"
_ledger.append(1)
assert hasattr(inspect, "Signature"), "exposes Signature"
_ledger.append(1)

# 2. isclass / isfunction on real targets.
assert inspect.isclass(_TargetClass) == True, "isclass(class) is True"
_ledger.append(1)
assert inspect.isclass(_target_no_defaults) == False, "isclass(fn) is False"
_ledger.append(1)
assert inspect.isfunction(_target_no_defaults) == True, "isfunction(fn) is True"
_ledger.append(1)
assert inspect.isfunction(_TargetClass) == False, "isfunction(class) is False"
_ledger.append(1)
assert inspect.isfunction(inspect) == False, "isfunction(module) is False"
_ledger.append(1)

# 3. signature() — parameter names, defaults, kinds.
_sig = inspect.signature(_target_with_defaults)
_params = _sig.parameters
assert "x" in _params, "signature parameters include 'x'"
_ledger.append(1)
assert "y" in _params, "signature parameters include 'y'"
_ledger.append(1)
assert "z" in _params, "signature parameters include 'z'"
_ledger.append(1)
# Boxed-int dodge for length check.
_param_names = list(_params.keys())
assert len(_param_names) - 3 == 0, "signature() lists 3 parameters"
_ledger.append(1)
# Default values
assert _params["y"].default == 10, "y default is 10"
_ledger.append(1)
assert _params["z"].default == "hi", "z default is 'hi'"
_ledger.append(1)
# 'x' has no default — sentinel is Parameter.empty.
assert _params["x"].default is inspect.Parameter.empty, "x has no default (Parameter.empty)"
_ledger.append(1)

# 4. getfullargspec — args / defaults / varargs / varkw.
_spec = inspect.getfullargspec(_target_with_defaults)
assert isinstance(_spec.args, list), "getfullargspec.args is a list"
_ledger.append(1)
assert _spec.args[0] == "x", "args[0] == 'x'"
_ledger.append(1)
assert _spec.args[1] == "y", "args[1] == 'y'"
_ledger.append(1)
assert _spec.args[2] == "z", "args[2] == 'z'"
_ledger.append(1)
# defaults tuple corresponds to the trailing positionals (y, z).
assert _spec.defaults == (10, "hi"), "defaults tuple == (10, 'hi')"
_ledger.append(1)
# No *args / **kwargs on this target.
assert _spec.varargs is None, "varargs is None on non-varargs fn"
_ledger.append(1)
assert _spec.varkw is None, "varkw is None on non-varkw fn"
_ledger.append(1)

# Varargs target — varargs / varkw populated.
_spec_va = inspect.getfullargspec(_target_varargs)
assert _spec_va.varargs == "args", "varargs == 'args' on *args fn"
_ledger.append(1)
assert _spec_va.varkw == "kwargs", "varkw == 'kwargs' on **kwargs fn"
_ledger.append(1)

# 5. getmembers — predicate-filtered listing.
_methods = inspect.getmembers(_TargetClass, inspect.isfunction)
assert isinstance(_methods, list), "getmembers() returns a list"
_ledger.append(1)
# Each entry is (name, value). Build a dict for stable assertions.
_member_dict = dict(_methods)
assert "method" in _member_dict, "getmembers filters in 'method'"
_ledger.append(1)
# Unfiltered getmembers exposes the class attribute too.
_all_members = dict(inspect.getmembers(_TargetClass))
assert "attr" in _all_members, "unfiltered getmembers exposes 'attr'"
_ledger.append(1)
assert _all_members["attr"] == 42, "attr value preserved through getmembers"
_ledger.append(1)

# 6. getsourcelines — non-empty list + integer line number.
_lines, _lineno = inspect.getsourcelines(_target_no_defaults)
assert isinstance(_lines, list), "getsourcelines returns (list, int)"
_ledger.append(1)
assert len(_lines) > 0, "getsourcelines returns non-empty source"
_ledger.append(1)
assert isinstance(_lineno, int), "getsourcelines line number is int"
_ledger.append(1)
assert _lineno > 0, "getsourcelines line number is positive"
_ledger.append(1)

# 7. Parameter sentinels — kinds enum members.
assert hasattr(inspect.Parameter, "POSITIONAL_OR_KEYWORD"), "Parameter.POSITIONAL_OR_KEYWORD exists"
_ledger.append(1)
assert hasattr(inspect.Parameter, "VAR_POSITIONAL"), "Parameter.VAR_POSITIONAL exists"
_ledger.append(1)
assert hasattr(inspect.Parameter, "VAR_KEYWORD"), "Parameter.VAR_KEYWORD exists"
_ledger.append(1)
# Varargs signature — confirm kinds line up with the target shape.
_sig_va = inspect.signature(_target_varargs)
_pkinds = [p.kind for p in _sig_va.parameters.values()]
assert inspect.Parameter.VAR_POSITIONAL in _pkinds, "*args param kind == VAR_POSITIONAL"
_ledger.append(1)
assert inspect.Parameter.VAR_KEYWORD in _pkinds, "**kwargs param kind == VAR_KEYWORD"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: inspect {len(_ledger)} asserts")
