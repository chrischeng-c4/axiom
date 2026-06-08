# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_percent_format_width_precision_mapping_ops"
# subject = "cpython321.test_percent_format_width_precision_mapping_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_percent_format_width_precision_mapping_ops.py"
# status = "filled"
# ///
"""cpython321.test_percent_format_width_precision_mapping_ops: execute CPython 3.12 seed test_percent_format_width_precision_mapping_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `%`-format operator's valid
# (matching) surface — the canonical width / precision / flag /
# conversion / multi-arg-tuple / mapping forms that the existing
# `lang_format_specs.py` does NOT exhaustively cover (it asserts only
# `"%d" % 42`, `"%s" % "abc"`, and `"%s is %d" % ("x", 5)`).
# Surface (the matching subset between mamba and CPython):
#   • single-arg integer codes — %s, %d, %x, %X, %o on int / str /
#     None / negative / zero;
#   • flag-width-precision — %05d zero-pad, %+d explicit sign, % d
#     space-sign, %-10s left-align, %10s right-align, %.3f precision;
#   • character code — %c with int (codepoint) and 1-char str;
#   • multi-arg tuple — %s %s, %d %d, %s=%d, %d.%d;
#   • mapping form — %(name)s, %(a)s-%(b)s, %(n)d;
#   • literal % escape — %%, 100%%;
#   • mixed codes — %s %d %f, %s = %r;
#   • %r conversion on str / list / None / int;
#   • multi-element mapping with int / str values.
_ledger: list[int] = []

# Single-arg %s
assert "%s" % "foo" == "foo"; _ledger.append(1)
assert "%s" % 5 == "5"; _ledger.append(1)
assert "%s" % None == "None"; _ledger.append(1)
assert "%s" % True == "True"; _ledger.append(1)
assert "%s" % [1, 2] == "[1, 2]"; _ledger.append(1)

# Single-arg %d on int (positive / negative / zero)
assert "%d" % 5 == "5"; _ledger.append(1)
assert "%d" % -5 == "-5"; _ledger.append(1)
assert "%d" % 0 == "0"; _ledger.append(1)
assert "%d" % 999 == "999"; _ledger.append(1)

# %x / %X / %o base-conversion
assert "%x" % 255 == "ff"; _ledger.append(1)
assert "%X" % 255 == "FF"; _ledger.append(1)
assert "%o" % 8 == "10"; _ledger.append(1)
assert "%x" % 0 == "0"; _ledger.append(1)
assert "%o" % 0 == "0"; _ledger.append(1)

# Flag-width-precision
assert "%05d" % 42 == "00042"; _ledger.append(1)
assert "%05d" % 1 == "00001"; _ledger.append(1)
assert "%+d" % 42 == "+42"; _ledger.append(1)
assert "%+d" % -42 == "-42"; _ledger.append(1)
assert "%+d" % 0 == "+0"; _ledger.append(1)
assert "% d" % 42 == " 42"; _ledger.append(1)
assert "% d" % -42 == "-42"; _ledger.append(1)

# Width / alignment on strings
assert "%-10s" % "hi" == "hi        "; _ledger.append(1)
assert "%10s" % "hi" == "        hi"; _ledger.append(1)
assert "%-5s|" % "x" == "x    |"; _ledger.append(1)
assert "|%5s" % "x" == "|    x"; _ledger.append(1)

# Float precision
assert "%.3f" % 3.14159 == "3.142"; _ledger.append(1)
assert "%.0f" % 3.14159 == "3"; _ledger.append(1)
assert "%.2f" % 3.0 == "3.00"; _ledger.append(1)

# %c character code (int + 1-char str)
assert "%c" % 65 == "A"; _ledger.append(1)
assert "%c" % 97 == "a"; _ledger.append(1)
assert "%c" % 48 == "0"; _ledger.append(1)
assert "%c" % "A" == "A"; _ledger.append(1)
assert "%c" % "z" == "z"; _ledger.append(1)

# Multi-arg tuple — positional
assert "%s %s" % (1, 2) == "1 2"; _ledger.append(1)
assert "%d %d" % (1, 2) == "1 2"; _ledger.append(1)
assert "%s=%d" % ("count", 5) == "count=5"; _ledger.append(1)
assert "%d.%d" % (3, 14) == "3.14"; _ledger.append(1)
assert "%s-%s-%s" % ("a", "b", "c") == "a-b-c"; _ledger.append(1)
assert "%d+%d=%d" % (1, 2, 3) == "1+2=3"; _ledger.append(1)

# Mapping form — by name
assert "%(name)s" % {"name": "X"} == "X"; _ledger.append(1)
assert "%(a)s-%(b)s" % {"a": 1, "b": 2} == "1-2"; _ledger.append(1)
assert "%(n)d" % {"n": 42} == "42"; _ledger.append(1)
assert "%(k)s=%(v)s" % {"k": "color", "v": "red"} == "color=red"; _ledger.append(1)

# Literal %% — escaped percent
assert "100%%" % () == "100%"; _ledger.append(1)
assert "%%d" % () == "%d"; _ledger.append(1)
assert "%%%%" % () == "%%"; _ledger.append(1)

# Mixed format codes
assert "%s %d %f" % ("a", 1, 2.0) == "a 1 2.000000"; _ledger.append(1)
assert "%s = %r" % ("x", 5) == "x = 5"; _ledger.append(1)
assert "%s/%d/%s" % ("a", 5, "b") == "a/5/b"; _ledger.append(1)

# %r conversion
assert "%r" % "abc" == "'abc'"; _ledger.append(1)
assert "%r" % [1, 2] == "[1, 2]"; _ledger.append(1)
assert "%r" % None == "None"; _ledger.append(1)
assert "%r" % 5 == "5"; _ledger.append(1)
assert "%r" % True == "True"; _ledger.append(1)

# Identity / round-trip with no codes
assert "no codes here" % () == "no codes here"; _ledger.append(1)
assert "" % () == ""; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_percent_format_width_precision_mapping_ops {sum(_ledger)} asserts")
