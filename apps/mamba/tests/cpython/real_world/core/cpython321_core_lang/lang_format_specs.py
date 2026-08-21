# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_format_specs"
# subject = "cpython321.lang_format_specs"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_format_specs.py"
# status = "filled"
# ///
"""cpython321.lang_format_specs: execute CPython 3.12 seed lang_format_specs"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for Python's three string-format
# surfaces — %-formatting, f-strings, and str.format / builtin format().
# Surface: %d / %s / %f (with .precision); %x / %X / %o; %5d width
# padding; %05d zero-pad; multi-arg %-tuple; f-string {n} basic /
# {n:5} width / {n:05} zero-fill / {pi:.2f} precision; align
# specifiers `<` / `>` / `^` with explicit fill char; base-conversion
# specifiers `x` / `X` / `b` / `o` / `e` (exponential); conversion
# flags `!r` and `!s` route through __repr__ / __str__; computed
# expressions inside f-string braces; format(value, spec) builtin;
# str.format with positional, named, and explicitly-indexed fields
# (incl. reuse like "{0} {1} {0}"); repr() / str() for ints and strs.
_ledger: list[int] = []

# %-formatting — basic types
assert "%d" % 42 == "42"; _ledger.append(1)
assert "%s" % "abc" == "abc"; _ledger.append(1)
assert "%.2f" % 3.14159 == "3.14"; _ledger.append(1)

# %-formatting — width and zero-pad
assert "%5d" % 42 == "   42"; _ledger.append(1)
assert "%05d" % 42 == "00042"; _ledger.append(1)

# %-formatting — multi-arg tuple
assert "%s is %d" % ("x", 5) == "x is 5"; _ledger.append(1)

# %-formatting — alternate bases
assert "%x" % 255 == "ff"; _ledger.append(1)
assert "%X" % 255 == "FF"; _ledger.append(1)
assert "%o" % 8 == "10"; _ledger.append(1)

# f-string — basic interpolation
n = 42
assert f"{n}" == "42"; _ledger.append(1)
# Width
assert f"{n:5}" == "   42"; _ledger.append(1)
# Zero-fill
assert f"{n:05}" == "00042"; _ledger.append(1)
# Float precision
pi = 3.14159
assert f"{pi:.2f}" == "3.14"; _ledger.append(1)

# f-string — alignment specifiers
assert f"{n:<5}" == "42   "; _ledger.append(1)
assert f"{n:>5}" == "   42"; _ledger.append(1)
assert f"{n:^5}" == " 42  "; _ledger.append(1)
# Explicit fill char
assert f"{n:*<5}" == "42***"; _ledger.append(1)
assert f"{n:*>5}" == "***42"; _ledger.append(1)
assert f"{n:-^6}" == "--42--"; _ledger.append(1)

# f-string — base-conversion specifiers
assert f"{255:x}" == "ff"; _ledger.append(1)
assert f"{255:X}" == "FF"; _ledger.append(1)
assert f"{5:b}" == "101"; _ledger.append(1)
assert f"{8:o}" == "10"; _ledger.append(1)
# Exponential notation
assert f"{1500000:.2e}" == "1.50e+06"; _ledger.append(1)

# f-string — !r and !s conversion flags
class _C:
    def __repr__(self) -> str: return "REPR"
    def __str__(self) -> str: return "STR"
_c = _C()
assert f"{_c!r}" == "REPR"; _ledger.append(1)
assert f"{_c!s}" == "STR"; _ledger.append(1)

# f-string — computed expressions inside braces
x = 5
y = 3
assert f"{x + y}" == "8"; _ledger.append(1)
assert f"{[1, 2, 3][0]}" == "1"; _ledger.append(1)
assert f"{x * 2 + 1}" == "11"; _ledger.append(1)

# format() builtin
assert format(42, "05d") == "00042"; _ledger.append(1)
assert format(3.14159, ".2f") == "3.14"; _ledger.append(1)
# Percent format
assert format(0.5, ".0%") == "50%"; _ledger.append(1)

# str.format — positional, named, indexed (with reuse)
assert "{}-{}".format(1, 2) == "1-2"; _ledger.append(1)
assert "{name}".format(name="x") == "x"; _ledger.append(1)
assert "{0} {1} {0}".format("a", "b") == "a b a"; _ledger.append(1)
assert "{1} {0}".format("a", "b") == "b a"; _ledger.append(1)

# repr() and str() for primitive types
assert repr(42) == "42"; _ledger.append(1)
assert repr("ab") == "'ab'"; _ledger.append(1)
assert str(42) == "42"; _ledger.append(1)
assert str(3.14) == "3.14"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_format_specs {sum(_ledger)} asserts")
