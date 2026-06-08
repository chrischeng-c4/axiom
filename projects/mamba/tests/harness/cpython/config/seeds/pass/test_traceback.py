# test_traceback.py — #3436 axis-1 stdlib traceback AssertionPass seed.
#
# Mamba-authored seed exercising the `traceback` module surface called
# out in the issue: format_exc, extract_tb, format_exception_only,
# walk_tb, TracebackException.
#
# Surface coverage (asserts run at module scope; no helper closures, per
# the mamba top-level def() quirk documented in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. traceback.format_exc()              — current exception → str
#                                            including type, message,
#                                            'Traceback' header.
#   3. traceback.format_exception_only()   — single-exception formatter
#                                            (type + value, no tb).
#   4. traceback.extract_tb()              — traceback object → list
#                                            of FrameSummary records.
#   5. traceback.walk_tb()                 — traceback object →
#                                            iterator of (frame, lineno).
#   6. traceback.TracebackException        — constructor + .format()
#                                            captures type+value+tb.
#
# Boxed-int dodge: length / lineno comparisons use subtraction-against-
# zero where applicable, per the boxed-accumulator equality bug.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: traceback N asserts` to stdout, which the
#     runner's ASSERTION_PASS_MARKERS lookup escalates to AssertionPass.

import traceback

_ledger: list[int] = []

# 1. Module identity + public surface.
assert traceback.__name__ == "traceback", "traceback.__name__ == 'traceback'"
_ledger.append(1)
assert hasattr(traceback, "format_exc"), "traceback exposes format_exc"
_ledger.append(1)
assert hasattr(traceback, "format_exception_only"), (
    "traceback exposes format_exception_only"
)
_ledger.append(1)
assert hasattr(traceback, "extract_tb"), "traceback exposes extract_tb"
_ledger.append(1)
assert hasattr(traceback, "walk_tb"), "traceback exposes walk_tb"
_ledger.append(1)
assert hasattr(traceback, "TracebackException"), (
    "traceback exposes TracebackException"
)
_ledger.append(1)
assert hasattr(traceback, "FrameSummary"), "traceback exposes FrameSummary"
_ledger.append(1)

# 2. format_exc — current exception formatted as str.
_formatted = ""
try:
    raise ValueError("boom-format-exc")
except ValueError:
    _formatted = traceback.format_exc()
assert isinstance(_formatted, str), "format_exc returns str"
_ledger.append(1)
assert "ValueError" in _formatted, "format_exc includes exception type name"
_ledger.append(1)
assert "boom-format-exc" in _formatted, "format_exc includes exception message"
_ledger.append(1)
assert "Traceback" in _formatted, "format_exc emits the 'Traceback' header"
_ledger.append(1)

# 3. format_exception_only — single-exception formatter (type + value).
_only_lines = traceback.format_exception_only(
    ValueError, ValueError("boom-only")
)
assert isinstance(_only_lines, list), "format_exception_only returns list"
_ledger.append(1)
assert len(_only_lines) - 0 > 0, "format_exception_only returns non-empty list"
_ledger.append(1)
_only_joined = "".join(_only_lines)
assert "ValueError" in _only_joined, (
    "format_exception_only output names the exception type"
)
_ledger.append(1)
assert "boom-only" in _only_joined, (
    "format_exception_only output carries the exception message"
)
_ledger.append(1)

# 4. extract_tb — traceback object → list of FrameSummary entries.
_extracted = []
try:
    raise RuntimeError("boom-extract")
except RuntimeError as _e_extract:
    _extracted = traceback.extract_tb(_e_extract.__traceback__)
assert isinstance(_extracted, list), "extract_tb returns a list"
_ledger.append(1)
assert len(_extracted) - 0 > 0, (
    "extract_tb produces at least one FrameSummary from the caught exception"
)
_ledger.append(1)
_first_frame = _extracted[0]
assert hasattr(_first_frame, "name"), "FrameSummary exposes .name"
_ledger.append(1)
assert hasattr(_first_frame, "filename"), "FrameSummary exposes .filename"
_ledger.append(1)
assert hasattr(_first_frame, "lineno"), "FrameSummary exposes .lineno"
_ledger.append(1)
assert isinstance(_first_frame.filename, str), (
    "FrameSummary.filename is a str"
)
_ledger.append(1)

# 5. walk_tb — traceback object → iterator of (frame, lineno) pairs.
_walked = []
try:
    raise KeyError("boom-walk")
except KeyError as _e_walk:
    _walked = list(traceback.walk_tb(_e_walk.__traceback__))
assert isinstance(_walked, list), "walk_tb materializes to a list"
_ledger.append(1)
assert len(_walked) - 0 > 0, "walk_tb yields at least one frame pair"
_ledger.append(1)
_walked_first = _walked[0]
assert isinstance(_walked_first, tuple), "walk_tb yields tuples"
_ledger.append(1)
assert len(_walked_first) - 2 == 0, "walk_tb tuple has 2 elements (frame, lineno)"
_ledger.append(1)

# 6. TracebackException — class capturing type+value+tb; .format() emits
#    a list of strings reconstructing the formatted traceback.
_te_lines = []
try:
    raise TypeError("boom-te")
except TypeError as _e_te:
    _te = traceback.TracebackException(
        type(_e_te), _e_te, _e_te.__traceback__
    )
    _te_lines = list(_te.format())
assert isinstance(_te_lines, list), "TracebackException.format() returns a list"
_ledger.append(1)
assert len(_te_lines) - 0 > 0, "TracebackException.format() yields lines"
_ledger.append(1)
_te_joined = "".join(_te_lines)
assert "TypeError" in _te_joined, (
    "TracebackException.format() output names the exception type"
)
_ledger.append(1)
assert "boom-te" in _te_joined, (
    "TracebackException.format() output carries the exception message"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: traceback {len(_ledger)} asserts")
