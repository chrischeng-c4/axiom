# Operational AssertionPass seed for SILENT divergences across the
# fuzzy-diff surface pinned by atomic 166: `difflib` (the
# documented `SequenceMatcher` instance constructor + `.ratio` /
# `.get_matching_blocks` instance methods, the documented
# `ndiff` / `Differ` / `context_diff` attribute surface, and
# the documented `unified_diff` line-level diff generator).
#
# The matching subset (math deeper helper layer + module
# hasattr for inf / nan / tau, cmath full layer + module
# hasattr surface, str.maketrans + translate full surface,
# difflib.get_close_matches fuzzy-match helper, difflib module
# hasattr surface for SequenceMatcher / get_close_matches /
# unified_diff) is covered by
# `test_math_cmath_maketrans_difflib_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • difflib.SequenceMatcher(None, "abcd", "abce") returns a
#     SequenceMatcher instance whose `.ratio()` returns 0.75
#     (mamba: constructor returns a bare float — type(sm) is
#     `float` — so `sm.ratio()` raises AttributeError, 'float'
#     object has no attribute 'ratio');
#   • difflib.SequenceMatcher(None, "abc", "abc").ratio() ==
#     1.0 — identical-input ratio contract (mamba: same
#     AttributeError);
#   • hasattr(difflib, "ndiff") is True — documented line-
#     level diff helper (mamba: False);
#   • hasattr(difflib, "Differ") is True — documented Differ
#     class identifier (mamba: False);
#   • hasattr(difflib, "context_diff") is True — documented
#     context-diff helper (mamba: False);
#   • list(difflib.unified_diff(["a\n","b\n"], ["a\n","c\n"],
#     lineterm="")) returns a 6-element diff output (mamba:
#     returns the empty list — generator never yields).
import difflib as _difflib_mod
from typing import Any

# Module binding retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
difflib: Any = _difflib_mod


_ledger: list[int] = []

# 1) difflib.SequenceMatcher — instance constructor + .ratio
_sm = difflib.SequenceMatcher(None, "abcd", "abce")
assert _sm.ratio() == 0.75; _ledger.append(1)

_sm2 = difflib.SequenceMatcher(None, "abc", "abc")
assert _sm2.ratio() == 1.0; _ledger.append(1)

# 2) difflib — documented helper attribute surface
assert hasattr(difflib, "ndiff") == True; _ledger.append(1)
assert hasattr(difflib, "Differ") == True; _ledger.append(1)
assert hasattr(difflib, "context_diff") == True; _ledger.append(1)

# 3) difflib.unified_diff — line-level diff generator
_ud = list(difflib.unified_diff(["a\n", "b\n"], ["a\n", "c\n"], lineterm=""))
assert _ud == ["--- ", "+++ ", "@@ -1,2 +1,2 @@", " a\n", "-b\n", "+c\n"]; _ledger.append(1)
assert len(_ud) == 6; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_difflib_sequencematcher_silent {sum(_ledger)} asserts")
