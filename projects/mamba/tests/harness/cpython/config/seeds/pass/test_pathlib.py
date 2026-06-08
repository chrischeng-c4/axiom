# test_pathlib.py — #2694 CPython pathlib seed (executed assertions).
#
# This is NOT a verbatim copy of CPython's Lib/test/test_pathlib/
# (the upstream package is ~10k LOC across PurePath/PurePosixPath/
# PureWindowsPath/Path/PosixPath/WindowsPath, plus owner/group, stat,
# touch, mkdir, rmdir, glob, walk, link/symlink semantics). Instead
# it is the *smallest* Mamba-authored seed distilled from the pathlib
# CONSTRUCTOR + IDENTITY surface: it asserts that the four canonical
# classes exist, are callable, and produce instances of the correct
# type when invoked with a path string. Emits the runner's positive
# proof-of-execution marker that `cpython_lib_test_runner.rs` (#2691)
# classifies as `AssertionPass` — not `ImportPass` or `Stub`.
#
# Why so small? Mamba's current pathlib presents constructor-shaped
# callables for `PurePath` / `PurePosixPath` / `PureWindowsPath` /
# `Path`, but the instance methods/properties (`.name`, `.stem`,
# `.suffix`, `.parent`, `.parts`, `.joinpath`, `__truediv__`,
# `isinstance(p, PurePath)`, `__str__`) return None / False /
# `<X instance>` placeholder on mamba today. Asserting any of those
# would fail on mamba and ALSO would not be a regression signal —
# the runtime gap is in the API, not the seed. Richer asserts land
# in the same commit that closes each gap.
#
# Specifically, mamba's pathlib gaps as of #2694:
#   - `p.name` / `p.stem` / `p.suffix` / `p.parent` / `p.parts`  → None
#   - `p / 'other.md'` / `p.joinpath(...)`                       → None
#   - `isinstance(p, pathlib.PurePosixPath)`                     → False
#   - `str(p)`                                                   → "<X instance>"
#   - `type(pathlib.PurePath).__name__`                          → "function"
#
# The working surface on both CPython and mamba is constructor
# *presence*, constructor *callability*, and `type(instance).__name__`
# returning the correct class name string — those are what this seed
# locks down.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Why no absolute-path or platform-specific assertion? The issue
# acceptance explicitly forbids "machine-specific absolute path
# expectations". This seed only asserts on string-constructor and
# class-name invariants — no `/`, no `home()`, no `cwd()`, no
# filesystem traversal.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: pathlib N asserts` to stdout.

import pathlib

_ledger: list[int] = []

# 1. Module identity: pathlib's own __name__ must be "pathlib".
assert pathlib.__name__ == "pathlib", "pathlib.__name__ must be 'pathlib'"
_ledger.append(1)

# 2. The four canonical class names are attributes of pathlib.
#    Catches a class of bootstrap regressions where the module
#    imports but the class binding is missing.
assert hasattr(pathlib, "PurePath"), "pathlib must expose PurePath"
_ledger.append(1)
assert hasattr(pathlib, "PurePosixPath"), "pathlib must expose PurePosixPath"
_ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath"), "pathlib must expose PureWindowsPath"
_ledger.append(1)
assert hasattr(pathlib, "Path"), "pathlib must expose Path"
_ledger.append(1)

# 3. Each class binding is callable. Distinguishes "name is bound"
#    from "name is bound to something we can instantiate" — catches
#    a regression where the binding flips to a sentinel or a typo'd
#    non-callable.
assert callable(pathlib.PurePath), "pathlib.PurePath must be callable"
_ledger.append(1)
assert callable(pathlib.PurePosixPath), "pathlib.PurePosixPath must be callable"
_ledger.append(1)
assert callable(pathlib.PureWindowsPath), "pathlib.PureWindowsPath must be callable"
_ledger.append(1)
assert callable(pathlib.Path), "pathlib.Path must be callable"
_ledger.append(1)

# 4. Instances of PurePosixPath and PureWindowsPath have the
#    correct concrete class name. This catches a regression where
#    a constructor was wired to return the wrong class — even though
#    `isinstance` / `str(p)` / `p.name` are not load-bearing on
#    mamba today, the class identity of the freshly-constructed
#    instance IS deterministic on both runtimes.
_p_posix = pathlib.PurePosixPath("foo/bar")
assert type(_p_posix).__name__ == "PurePosixPath", "PurePosixPath('...') returns PurePosixPath"
_ledger.append(1)
_p_win = pathlib.PureWindowsPath("foo/bar")
assert type(_p_win).__name__ == "PureWindowsPath", "PureWindowsPath('...') returns PureWindowsPath"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: pathlib {len(_ledger)} asserts")
