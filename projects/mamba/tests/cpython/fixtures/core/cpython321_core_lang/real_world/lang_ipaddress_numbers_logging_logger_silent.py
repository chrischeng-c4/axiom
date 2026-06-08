# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_ipaddress_numbers_logging_logger_silent"
# subject = "cpython321.lang_ipaddress_numbers_logging_logger_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_ipaddress_numbers_logging_logger_silent.py"
# status = "filled"
# ///
"""cpython321.lang_ipaddress_numbers_logging_logger_silent: execute CPython 3.12 seed lang_ipaddress_numbers_logging_logger_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in
# `ipaddress` (the IPv4Address / IPv6Address / IPv4Network /
# ip_address constructors plus the documented `.is_private`
# predicate), `numbers` (the abstract base classes Number / Real /
# Integral / Complex / Rational plus the `isinstance(int, Number)`
# / `isinstance(int, Integral)` / `isinstance(float, Real)`
# contracts), `shlex` (split honouring single-quoted multi-word
# tokens), `platform` (the architecture / python_implementation /
# uname helpers), `itertools` (chain.from_iterable producing the
# concatenation of the nested iterables), and `logging` (the
# NOTSET sentinel, getLevelName, getLogger returning a real
# Logger instance, and Logger / Handler / StreamHandler class
# identity).
#
# The matching subset (shlex.split on unquoted input + shlex.quote
# / join, getopt.getopt for both short and long-option syntax,
# itertools.islice / takewhile / dropwhile / filterfalse / compress
# / starmap / accumulate / pairwise, logging integer level
# constants, platform.system / python_version / machine returning
# `str`) is covered by `test_shlex_getopt_itertools_logging_ops`;
# this fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • ipaddress.IPv4Address("192.168.1.1") — instance constructor
#     (mamba: AttributeError, ipaddress is a `dict` with no
#     `IPv4Address` key);
#   • type(ipaddress.IPv4Address(...)).__name__ == "IPv4Address";
#   • str(ipaddress.IPv4Address("192.168.1.1")) == "192.168.1.1";
#   • int(ipaddress.IPv4Address("192.168.1.1")) == 3232235777;
#   • ipaddress.IPv4Network("192.168.1.0/24").num_addresses == 256;
#   • ipaddress.IPv4Network("192.168.1.0/24").prefixlen == 24;
#   • ipaddress.IPv6Address("::1") — IPv6 constructor;
#   • ipaddress.ip_address("1.2.3.4") returns an IPv4Address;
#   • numbers.Number.__name__ == "Number" — class identity (mamba:
#     returns None, the binding is a `<lambda>`);
#   • numbers.Real / Integral / Complex / Rational class identity;
#   • isinstance(1, numbers.Number) is True — int is a Number
#     (mamba: returns False);
#   • isinstance(1, numbers.Integral) is True (mamba: False);
#   • isinstance(1.5, numbers.Real) is True (mamba: False);
#   • shlex.split("hello 'world foo' bar") ==
#     ["hello", "world foo", "bar"] — single quotes form one token
#     (mamba: returns ["hello", "'world", "foo'", "bar"]);
#   • platform.architecture() — returns a 2-tuple (mamba:
#     AttributeError);
#   • platform.python_implementation() == "CPython" on CPython
#     (mamba: AttributeError);
#   • itertools.chain.from_iterable([[1,2], [3,4]]) ->
#     [1, 2, 3, 4] (mamba: returns []);
#   • logging.NOTSET == 0 — sentinel level integer (mamba: None);
#   • logging.getLevelName(20) == "INFO" — level-int -> name
#     mapping (mamba: AttributeError);
#   • type(logging.getLogger("test")).__name__ == "Logger" —
#     class identity of the returned logger (mamba: returns a
#     `dict`);
#   • logging.Logger.__name__ == "Logger" — class identity (mamba:
#     None);
#   • logging.Handler.__name__ == "Handler" (mamba: None);
#   • logging.StreamHandler.__name__ == "StreamHandler" (mamba:
#     None).
import ipaddress as _ipaddress_mod
import numbers as _numbers_mod
import shlex as _shlex_mod
import platform as _platform_mod
import itertools as _itertools_mod
import logging as _logging_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constructors / class identifiers / module-level helpers that
# mamba's bundled type stubs do not surface.
ipaddress: Any = _ipaddress_mod
numbers: Any = _numbers_mod
shlex: Any = _shlex_mod
platform: Any = _platform_mod
itertools: Any = _itertools_mod
logging: Any = _logging_mod

_ledger: list[int] = []

# 1) ipaddress.IPv4Address — class identity + str / int round-trip
_a4: Any = ipaddress.IPv4Address("192.168.1.1")
assert type(_a4).__name__ == "IPv4Address"; _ledger.append(1)
assert str(_a4) == "192.168.1.1"; _ledger.append(1)
assert int(_a4) == 3232235777; _ledger.append(1)

# 2) ipaddress.IPv4Network — num_addresses / prefixlen
_n4: Any = ipaddress.IPv4Network("192.168.1.0/24")
assert type(_n4).__name__ == "IPv4Network"; _ledger.append(1)
assert _n4.num_addresses == 256; _ledger.append(1)
assert _n4.prefixlen == 24; _ledger.append(1)

# 3) ipaddress.IPv6Address — class identity + str round-trip
_a6: Any = ipaddress.IPv6Address("::1")
assert type(_a6).__name__ == "IPv6Address"; _ledger.append(1)
assert str(_a6) == "::1"; _ledger.append(1)

# 4) ipaddress.ip_address — dispatch by string form
_aip: Any = ipaddress.ip_address("1.2.3.4")
assert type(_aip).__name__ == "IPv4Address"; _ledger.append(1)
assert str(_aip) == "1.2.3.4"; _ledger.append(1)

# 5) numbers.* — abstract base class identity
assert numbers.Number.__name__ == "Number"; _ledger.append(1)
assert numbers.Real.__name__ == "Real"; _ledger.append(1)
assert numbers.Integral.__name__ == "Integral"; _ledger.append(1)
assert numbers.Complex.__name__ == "Complex"; _ledger.append(1)
assert numbers.Rational.__name__ == "Rational"; _ledger.append(1)

# 6) numbers.* — isinstance contracts
assert isinstance(1, numbers.Number); _ledger.append(1)
assert isinstance(1, numbers.Integral); _ledger.append(1)
assert isinstance(1.5, numbers.Real); _ledger.append(1)

# 7) shlex.split — single-quoted multi-word tokens collapse
assert shlex.split("hello 'world foo' bar") == ["hello", "world foo", "bar"]; _ledger.append(1)

# 8) platform.architecture — returns a 2-tuple
_arch: Any = platform.architecture()
assert isinstance(_arch, tuple); _ledger.append(1)
assert len(_arch) == 2; _ledger.append(1)

# 9) platform.python_implementation — known string on CPython
assert platform.python_implementation() == "CPython"; _ledger.append(1)

# 10) itertools.chain.from_iterable — flatten nested iterables
assert list(itertools.chain.from_iterable([[1, 2], [3, 4]])) == [1, 2, 3, 4]; _ledger.append(1)
assert list(itertools.chain.from_iterable([["a"], ["b", "c"]])) == ["a", "b", "c"]; _ledger.append(1)

# 11) logging.NOTSET — sentinel level integer
assert logging.NOTSET == 0; _ledger.append(1)

# 12) logging.getLevelName — level-int -> name mapping
assert logging.getLevelName(20) == "INFO"; _ledger.append(1)
assert logging.getLevelName(30) == "WARNING"; _ledger.append(1)

# 13) logging.getLogger — returns a Logger instance
_lgr: Any = logging.getLogger("test")
assert type(_lgr).__name__ == "Logger"; _ledger.append(1)

# 14) logging.Logger / Handler / StreamHandler — class identity
assert logging.Logger.__name__ == "Logger"; _ledger.append(1)
assert logging.Handler.__name__ == "Handler"; _ledger.append(1)
assert logging.StreamHandler.__name__ == "StreamHandler"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ipaddress_numbers_logging_logger_silent {sum(_ledger)} asserts")
