# Operational AssertionPass seed for SILENT divergences across the
# bootstrap stdlib quintet pinned by atomic 143: `sys` (the
# documented platform / maxsize / version_info / maxunicode /
# stdin / stdout / stderr / copyright surface), `signal` (the
# Signals enum identity and Handlers enum identity plus the
# type-of-SIGINT / SIG_DFL class-identity contract), and
# `operator` (the attrgetter / methodcaller higher-order
# constructors that mamba lowers to None).
#
# The matching subset (sys.byteorder, sys.path / modules / argv /
# executable types, errno integer codes, signal portable integer
# sentinels + SIG_DFL / SIG_IGN / NSIG, stat S_IF*/S_I* full
# permission-mode constants + S_ISDIR / S_ISREG mode predicates,
# operator arithmetic / comparison / logical / bitwise / sequence-
# arity primitives) is covered by
# `test_sys_errno_signal_stat_operator_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • sys.platform == "darwin" on macOS (mamba: returns "macos");
#   • sys.maxsize == 9223372036854775807 — CPython i64 max
#     (mamba: returns 140737488355327, the 48-bit pointer-tag
#     ceiling per the [[project_mamba_runtime_correctness_gaps_
#     2026_05_13]] memo);
#   • sys.maxunicode == 1114111 — Unicode code-point ceiling
#     (mamba: returns None);
#   • type(sys.version_info).__name__ == "version_info" — named
#     tuple (mamba: returns "dict");
#   • type(sys.stdout).__name__ == "TextIOWrapper" (mamba:
#     returns "dict");
#   • type(sys.stderr).__name__ == "TextIOWrapper" (mamba: "dict");
#   • type(sys.stdin).__name__ == "TextIOWrapper" (mamba: "dict");
#   • hasattr(sys, "copyright") is True (mamba: False — the
#     binding is missing);
#   • type(signal.SIGINT).__name__ == "Signals" — Signals enum
#     member (mamba: returns "int", SIGINT is a plain integer);
#   • signal.Signals.__name__ == "Signals" — Signals enum class
#     identity (mamba: returns None);
#   • type(signal.SIG_DFL).__name__ == "Handlers" — Handlers enum
#     member (mamba: returns "int");
#   • signal.Handlers.__name__ == "Handlers" — Handlers enum
#     class identity (mamba: returns None);
#   • operator.attrgetter("upper")("hi") returns a bound-method
#     object (mamba: returns None);
#   • operator.methodcaller("upper")("hi") == "HI" — invokes the
#     named method (mamba: returns None).
import sys as _sys_mod
import signal as _signal_mod
import operator as _operator_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constants / class identifiers / higher-order constructors that
# mamba's bundled type stubs do not surface accurately.
sys: Any = _sys_mod
signal: Any = _signal_mod
operator: Any = _operator_mod

_ledger: list[int] = []

# 1) sys.platform — POSIX "darwin" sentinel on macOS
assert sys.platform == "darwin"; _ledger.append(1)

# 2) sys.maxsize — CPython i64 ceiling
assert sys.maxsize == 9223372036854775807; _ledger.append(1)

# 3) sys.maxunicode — Unicode code-point ceiling
assert sys.maxunicode == 1114111; _ledger.append(1)

# 4) sys.version_info — named-tuple type identity
assert type(sys.version_info).__name__ == "version_info"; _ledger.append(1)

# 5) sys.stdin / stdout / stderr — TextIOWrapper file objects
assert type(sys.stdout).__name__ == "TextIOWrapper"; _ledger.append(1)
assert type(sys.stderr).__name__ == "TextIOWrapper"; _ledger.append(1)
assert type(sys.stdin).__name__ == "TextIOWrapper"; _ledger.append(1)

# 6) sys.copyright — documented module-level binding
assert hasattr(sys, "copyright") == True; _ledger.append(1)

# 7) signal.Signals — enum-class identity + SIGINT enum member
assert type(signal.SIGINT).__name__ == "Signals"; _ledger.append(1)
assert signal.Signals.__name__ == "Signals"; _ledger.append(1)

# 8) signal.Handlers — enum-class identity + SIG_DFL enum member
assert type(signal.SIG_DFL).__name__ == "Handlers"; _ledger.append(1)
assert signal.Handlers.__name__ == "Handlers"; _ledger.append(1)

# 9) operator.attrgetter — bound-method-ref constructor
_ag = operator.attrgetter("upper")
assert _ag("hi") is not None; _ledger.append(1)

# 10) operator.methodcaller — invokes the named method
assert operator.methodcaller("upper")("hi") == "HI"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_sys_signal_operator_class_silent {sum(_ledger)} asserts")
