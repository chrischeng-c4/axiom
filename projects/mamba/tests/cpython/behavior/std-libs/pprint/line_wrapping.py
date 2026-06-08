# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "line_wrapping"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat breaks a composite only when its single-line render exceeds width; indent sets the continuation offset, compact bounds every line to width, and long str/bytes split across adjacent literals (round-tripping via eval)"""
import pprint

# indent controls continuation offset; width decides when to break.
o = [list(range(10)), dict(first=1, second=2, third=3)]
assert pprint.pformat(o, indent=4, width=42) == (
    "[   [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],\n"
    "    {'first': 1, 'second': 2, 'third': 3}]"
)
# A tighter width forces the inner dict to break, one key per line.
assert pprint.pformat(o, indent=4, width=41) == (
    "[   [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],\n"
    "    {   'first': 1,\n"
    "        'second': 2,\n"
    "        'third': 3}]"
)

# compact=True keeps the line width bound: every line stays within width
# (above the minimum needed to render the deepest single, unsplittable
# nesting frame).
deep = [10] * 10
for outer in range(19):
    deep = [deep]
for w in range(42, 69):
    lines = pprint.pformat(deep, width=w, compact=True).splitlines()
    assert max(len(line) for line in lines) <= w

# Long str values are split across adjacent string literals when narrow.
fox = "the quick brown fox jumped over a lazy dog"
assert pprint.pformat(fox, width=19) == (
    "('the quick brown '\n"
    " 'fox jumped over '\n"
    " 'a lazy dog')"
)

# Long bytes split the same way; round-trips back to the original.
letters = b"abcdefghijklmnopqrstuvwxyz"
assert pprint.pformat(letters, width=19) == (
    "(b'abcdefghijkl'\n b'mnopqrstuvwxyz')"
)
for width in range(1, 40):
    assert eval(pprint.pformat(fox, width=width)) == fox
    assert eval(pprint.pformat(letters, width=width)) == letters
print("line_wrapping OK")
