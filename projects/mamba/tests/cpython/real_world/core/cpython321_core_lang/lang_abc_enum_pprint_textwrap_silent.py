# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_abc_enum_pprint_textwrap_silent"
# subject = "cpython321.lang_abc_enum_pprint_textwrap_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_abc_enum_pprint_textwrap_silent.py"
# status = "filled"
# ///
"""cpython321.lang_abc_enum_pprint_textwrap_silent: execute CPython 3.12 seed lang_abc_enum_pprint_textwrap_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across
# the `abc.ABC` class metaclass identity contract +
# `enum` module identifier surface + `enum.Enum`-subclass
# class identity / member type / member `.value` /
# member `.name` / `len(class)` round-trip contract +
# `pprint` module identifier surface + `pprint.pformat`
# round-trip value contract + `textwrap` module
# identifier surface + `textwrap.dedent` / `.indent` /
# `.wrap` round-trip value contract pinned by atomic 201:
# `abc.ABC` (the documented `type(class(abc.ABC)).__name__
# == "ABCMeta"` metaclass identity contract — mamba
# collapses to "str" via the integer-handle pattern —
# and the documented `__abstractmethods__` attribute
# identifier on the abstract subclass — mamba: False),
# `enum` (the documented class / sentinel / decorator
# identifier surface — `ReprEnum` / `EnumMeta` /
# `FlagBoundary` / `STRICT` / `CONFORM` / `EJECT` /
# `KEEP` / `verify` / `EnumCheck` / `global_enum` /
# `member` / `nonmember` / `show_flag_values` / `property`),
# `enum.Enum`-subclass (the documented class identity,
# member type, member `.value`, member `.name`, and
# `len(class) == declared-member-count` round-trip
# contract — mamba collapses class to "str", member to
# "int", `.value` / `.name` to None, and `len()` to 5
# instead of 3), `pprint` (the documented class /
# helper identifier surface — `PrettyPrinter` / `pp` /
# `saferepr` / `isreadable` / `isrecursive`), `pprint`
# (the documented `pformat([1, 2, 3]) == "[1, 2, 3]"`
# single-line round-trip — mamba expands to multi-line),
# `textwrap` (the documented `TextWrapper` class
# identifier — mamba: False), and `textwrap`
# (the documented `.dedent` / `.indent` trailing-newline-
# preserving and `.wrap` multi-line-list round-trip —
# mamba strips trailing newlines and returns
# single-element list).
#
# The matching subset (full abc hasattr + ABC MRO
# contract, partial copy hasattr + shallow / deep
# round-trip identity, partial enum hasattr,
# partial pprint hasattr, partial textwrap hasattr)
# is covered by `test_abc_copy_enum_pprint_textwrap_value_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(class(abc.ABC)).__name__ == "ABCMeta" —
#     documented metaclass identity
#     (mamba: "str" via integer-handle pattern);
#   • hasattr(class(abc.ABC), "__abstractmethods__")
#     is True — documented attribute identifier
#     (mamba: False);
#   • hasattr(enum, "ReprEnum") is True — documented
#     class identifier (mamba: False);
#   • hasattr(enum, "EnumMeta") is True — documented
#     class identifier (mamba: False);
#   • hasattr(enum, "FlagBoundary") is True — documented
#     class identifier (mamba: False);
#   • hasattr(enum, "STRICT") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(enum, "CONFORM") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(enum, "EJECT") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(enum, "KEEP") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(enum, "verify") is True — documented
#     decorator identifier (mamba: False);
#   • hasattr(enum, "EnumCheck") is True — documented
#     class identifier (mamba: False);
#   • hasattr(enum, "global_enum") is True — documented
#     decorator identifier (mamba: False);
#   • hasattr(enum, "member") is True — documented
#     decorator identifier (mamba: False);
#   • hasattr(enum, "nonmember") is True — documented
#     decorator identifier (mamba: False);
#   • hasattr(enum, "show_flag_values") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(enum, "property") is True — documented
#     property identifier (mamba: False);
#   • type(enum.Enum-subclass).__name__ == "EnumType" —
#     documented class identity (mamba: "str");
#   • type(enum.Enum-subclass.MEMBER).__name__ ==
#     enum-subclass-name — documented member type
#     (mamba: "int");
#   • enum.Enum-subclass.MEMBER.value == declared-value
#     — documented value contract (mamba: None);
#   • enum.Enum-subclass.MEMBER.name == declared-name —
#     documented name contract (mamba: None);
#   • len(enum.Enum-subclass) == declared-count —
#     documented member-count contract (mamba: 5);
#   • hasattr(pprint, "PrettyPrinter") is True —
#     documented class identifier (mamba: False);
#   • hasattr(pprint, "pp") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(pprint, "saferepr") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(pprint, "isreadable") is True — documented
#     helper identifier (mamba: False);
#   • hasattr(pprint, "isrecursive") is True — documented
#     helper identifier (mamba: False);
#   • pprint.pformat([1, 2, 3]) == "[1, 2, 3]" —
#     documented single-line round-trip
#     (mamba: multi-line);
#   • hasattr(textwrap, "TextWrapper") is True —
#     documented class identifier (mamba: False);
#   • textwrap.dedent("    hello\n    world\n") ==
#     "hello\nworld\n" — documented trailing-newline-
#     preserving round-trip (mamba: strips trailing
#     newline);
#   • textwrap.indent("a\nb\n", "> ") == "> a\n> b\n"
#     — documented trailing-newline-preserving
#     round-trip (mamba: strips trailing newline);
#   • textwrap.wrap("a b c d e f", width=5) ==
#     ["a b c", "d e f"] — documented multi-line list
#     round-trip (mamba: returns single-element list).
import abc as _abc_mod
import enum as _enum_mod
import pprint as _pprint_mod
import textwrap as _textwrap_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute identifier behavior that
# mamba's bundled type stubs do not surface accurately.
abc: Any = _abc_mod
enum: Any = _enum_mod
pprint: Any = _pprint_mod
textwrap: Any = _textwrap_mod


class _Animal(abc.ABC):
    @abc.abstractmethod
    def sound(self): ...


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


# Class binding retyped as `Any` to bypass Pyright stub-driven
# narrowing of enum member `.value` / `.name` and `len(class)`
# attribute access — every spec contract below probes documented
# public enum class identity behavior that mamba's bundled type
# stubs do not surface accurately.
_ColorAny: Any = Color


_ledger: list[int] = []

# 1) abc.ABC — metaclass identity + abstract-method identifier
assert type(_Animal).__name__ == "ABCMeta"; _ledger.append(1)
assert hasattr(_Animal, "__abstractmethods__") == True; _ledger.append(1)

# 2) enum — module identifier surface
assert hasattr(enum, "ReprEnum") == True; _ledger.append(1)
assert hasattr(enum, "EnumMeta") == True; _ledger.append(1)
assert hasattr(enum, "FlagBoundary") == True; _ledger.append(1)
assert hasattr(enum, "STRICT") == True; _ledger.append(1)
assert hasattr(enum, "CONFORM") == True; _ledger.append(1)
assert hasattr(enum, "EJECT") == True; _ledger.append(1)
assert hasattr(enum, "KEEP") == True; _ledger.append(1)
assert hasattr(enum, "verify") == True; _ledger.append(1)
assert hasattr(enum, "EnumCheck") == True; _ledger.append(1)
assert hasattr(enum, "global_enum") == True; _ledger.append(1)
assert hasattr(enum, "member") == True; _ledger.append(1)
assert hasattr(enum, "nonmember") == True; _ledger.append(1)
assert hasattr(enum, "show_flag_values") == True; _ledger.append(1)
assert hasattr(enum, "property") == True; _ledger.append(1)

# 3) enum.Enum-subclass — class identity + member type +
#    member .value + member .name + len round-trip
assert type(_ColorAny).__name__ == "EnumType"; _ledger.append(1)
assert type(_ColorAny.RED).__name__ == "Color"; _ledger.append(1)
assert _ColorAny.RED.value == 1; _ledger.append(1)
assert _ColorAny.RED.name == "RED"; _ledger.append(1)
assert len(_ColorAny) == 3; _ledger.append(1)

# 4) pprint — module identifier surface
assert hasattr(pprint, "PrettyPrinter") == True; _ledger.append(1)
assert hasattr(pprint, "pp") == True; _ledger.append(1)
assert hasattr(pprint, "saferepr") == True; _ledger.append(1)
assert hasattr(pprint, "isreadable") == True; _ledger.append(1)
assert hasattr(pprint, "isrecursive") == True; _ledger.append(1)

# 5) pprint.pformat — single-line round-trip
assert pprint.pformat([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)

# 6) textwrap — module identifier surface
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

# 7) textwrap.dedent / .indent / .wrap round-trip
assert textwrap.dedent("    hello\n    world\n") == "hello\nworld\n"; _ledger.append(1)
assert textwrap.indent("a\nb\n", "> ") == "> a\n> b\n"; _ledger.append(1)
assert textwrap.wrap("a b c d e f", width=5) == ["a b c", "d e f"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_abc_enum_pprint_textwrap_silent {sum(_ledger)} asserts")
