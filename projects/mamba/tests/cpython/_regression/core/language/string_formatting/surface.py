"""Surface contract for language string formatting (f-strings, %, .format).

# type-regime: monomorphic

Probes: f-string basics, format spec, .format(), % formatting, repr/str/ascii
in f-strings, nested expressions, multiline f-strings.
CPython 3.12 is the oracle.
"""

# Basic f-string
_name = "world"
assert f"hello {_name}" == "hello world", f"basic f-string = {f'hello {_name}'!r}"

# Expression evaluation
_x = 7
assert f"{_x * 6}" == "42", f"expr = {f'{_x * 6}'!r}"

# Format spec — integer padding
assert f"{42:05d}" == "00042", f"pad = {f'{42:05d}'!r}"
assert f"{255:x}" == "ff", f"hex = {f'{255:x}'!r}"
assert f"{255:X}" == "FF", f"HEX = {f'{255:X}'!r}"
assert f"{255:08b}" == "11111111", f"bin = {f'{255:08b}'!r}"

# Format spec — float
assert f"{3.14159:.2f}" == "3.14", f"float2f = {f'{3.14159:.2f}'!r}"
assert f"{1234567.89:,.2f}" == "1,234,567.89", f"comma = {f'{1234567.89:,.2f}'!r}"

# !r, !s, !a conversions
_s = "it's alive"
_r = repr(_s)
assert f"{_s!r}" == _r, f"!r = {f'{_s!r}'!r}"
assert f"{_s!s}" == _s, f"!s = {f'{_s!s}'!r}"

# .format() method
assert "hello {}".format("world") == "hello world", ".format() positional"
assert "{name}!".format(name="Alice") == "Alice!", ".format() keyword"
assert "{0} {1} {0}".format("a", "b") == "a b a", ".format() repeat"

# % formatting
assert "%s %s" % ("hello", "world") == "hello world", "% string"
assert "%d + %d = %d" % (1, 2, 3) == "1 + 2 = 3", "% int"
assert "%.3f" % 3.14159 == "3.142", "% float"

# Nested f-string (expression containing another f-string)
_prefix = "x"
_n = 5
assert f"{'=' * _n}" == "=====", f"repeat in f-string = {f'{'=' * _n}'!r}"

# Multiline f-string (via concat)
_part1 = f"line1"
_part2 = f"line2"
assert _part1 + "\n" + _part2 == "line1\nline2", "multiline concat"

print("surface OK")
