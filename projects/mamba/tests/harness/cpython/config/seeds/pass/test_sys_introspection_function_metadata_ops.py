# Operational AssertionPass seed for the matching `sys` runtime-
# introspection surface + function/class `.__name__` metadata. No
# existing seed covers `sys`.
#
# This fixture exercises only the SHAPE / TYPE invariants of `sys`
# (every published attribute should exist with the documented type
# and within a sane numeric range) plus the user-visible
# `.__name__` attribute on def-functions and classes. The matching
# subset is deliberately narrow because mamba's `sys.platform`
# string differs ("macos" vs CPython "darwin"), `sys.maxsize` is a
# different bit width, `sys.version_info[i]` subscript fails, and
# both `sys.int_info`/`sys.hash_info` are plain dicts on mamba.
#
# Surface in this fixture:
#   • sys.platform — non-empty str;
#   • sys.maxsize — positive int with a reasonable lower bound;
#   • sys.byteorder — single member of {"little", "big"};
#   • sys.version_info.major / .minor — attribute-style access, with
#     major == 3;
#   • sys.float_info — has the IEEE-754-derived attributes (epsilon,
#     dig, mant_dig, max, min) within expected ranges;
#   • sys.executable — non-empty str path;
#   • sys.path — list of str entries;
#   • sys.modules — dict with at least one entry;
#   • sys.argv — list with at least one str element;
#   • sys.getrecursionlimit() — positive int;
#   • sys.intern("...") — returns the same string content;
#   • def-function and class `.__name__` — string with the source-
#     code identifier.
#
# Behavioral edges that DIVERGE on mamba (sys.platform == "darwin",
# sys.maxsize exact value, sys.version_info subscript, namedtuple
# identity of int_info / hash_info, setrecursionlimit no-op,
# lambda.__name__ == "<lambda>", built-in-type.__name__ returning a
# bound-method handle, sys.maxunicode == 0x10FFFF, Fraction/Decimal
# value identity) are covered in
# `lang_sys_platform_version_subscript_silent.py`.
import sys

_ledger: list[int] = []

# 1) sys.platform — non-empty str (exact value differs across mamba)
assert isinstance(sys.platform, str); _ledger.append(1)
assert len(sys.platform) > 0; _ledger.append(1)

# 2) sys.maxsize — positive int with a reasonable lower bound
assert isinstance(sys.maxsize, int); _ledger.append(1)
assert sys.maxsize > 0; _ledger.append(1)
# Sane lower bound for any 32-bit-or-bigger platform
assert sys.maxsize > (1 << 30); _ledger.append(1)

# 3) sys.byteorder
assert isinstance(sys.byteorder, str); _ledger.append(1)
assert sys.byteorder in ("little", "big"); _ledger.append(1)

# 4) sys.version_info — attribute-style access (subscript form
#    diverges and stays in the spec fixture)
assert isinstance(sys.version_info.major, int); _ledger.append(1)
assert isinstance(sys.version_info.minor, int); _ledger.append(1)
assert sys.version_info.major == 3; _ledger.append(1)
assert sys.version_info.minor >= 0; _ledger.append(1)

# 5) sys.float_info — IEEE-754 derived attributes
assert sys.float_info.epsilon > 0; _ledger.append(1)
assert sys.float_info.epsilon < 1; _ledger.append(1)
assert sys.float_info.dig == 15; _ledger.append(1)
assert sys.float_info.mant_dig == 53; _ledger.append(1)
assert sys.float_info.max > 1e100; _ledger.append(1)
assert sys.float_info.min > 0; _ledger.append(1)
assert sys.float_info.min < 1; _ledger.append(1)

# 6) sys.executable — non-empty str path
assert isinstance(sys.executable, str); _ledger.append(1)
assert len(sys.executable) > 0; _ledger.append(1)

# 7) sys.path — list of str entries
assert isinstance(sys.path, list); _ledger.append(1)
assert len(sys.path) > 0; _ledger.append(1)

# 8) sys.modules — dict with at least one entry
assert isinstance(sys.modules, dict); _ledger.append(1)
assert len(sys.modules) > 0; _ledger.append(1)
# `sys` itself is always in sys.modules
assert "sys" in sys.modules; _ledger.append(1)

# 9) sys.argv — list with at least one str element
assert isinstance(sys.argv, list); _ledger.append(1)
assert len(sys.argv) > 0; _ledger.append(1)
assert isinstance(sys.argv[0], str); _ledger.append(1)

# 10) sys.getrecursionlimit() — positive int
_rl = sys.getrecursionlimit()
assert isinstance(_rl, int); _ledger.append(1)
assert _rl > 0; _ledger.append(1)
# Sane lower bound — the limit is normally at least 100
assert _rl > 100; _ledger.append(1)

# 11) sys.intern("...") — returns the same string content
assert sys.intern("hello") == "hello"; _ledger.append(1)
assert isinstance(sys.intern("hello"), str); _ledger.append(1)
assert sys.intern("") == ""; _ledger.append(1)
assert sys.intern("the quick brown fox") == "the quick brown fox"; _ledger.append(1)

# 12) def-function and class .__name__
def my_named_function(a, b):
    return a + b

assert my_named_function.__name__ == "my_named_function"; _ledger.append(1)
assert isinstance(my_named_function.__name__, str); _ledger.append(1)

def another_one():
    pass

assert another_one.__name__ == "another_one"; _ledger.append(1)

class MyNamedClass:
    pass

assert MyNamedClass.__name__ == "MyNamedClass"; _ledger.append(1)
assert isinstance(MyNamedClass.__name__, str); _ledger.append(1)

class _Underscored:
    pass

assert _Underscored.__name__ == "_Underscored"; _ledger.append(1)

# NB: built-in types' .__name__ (int / str / list / dict / tuple /
# bool / float) returns an unbound-method handle on mamba instead of
# the type name — moved to the divergence-spec fixture. type(None)
# happens to match because NoneType is not a built-in type literal
# bound at parse time.
assert type(None).__name__ == "NoneType"; _ledger.append(1)

# NB: sys.maxunicode is None on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_sys_introspection_function_metadata_ops {sum(_ledger)} asserts")
