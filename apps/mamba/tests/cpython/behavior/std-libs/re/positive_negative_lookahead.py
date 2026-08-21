# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "positive_negative_lookahead"
# subject = "re.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.match: positive lookahead (?=...) constrains without consuming (r'a(?=\\d)' on 'a5' matches 'a', end 1); negative lookahead (?!...) blocks the forbidden follow"""
import re

# Positive lookahead: 'a' followed by a digit, the digit is not consumed.
m = re.match(r"a(?=\d)", "a5")
assert m is not None and m.group() == "a", f"lookahead group = {m.group()!r}"
assert m.end() == 1, f"lookahead end = {m.end()!r}"
# Negative lookahead: 'a' NOT followed by a digit.
assert re.match(r"a(?!\d)", "ab") is not None, "neg lookahead ok"
assert re.match(r"a(?!\d)", "a5") is None, "neg lookahead blocks digit"

print("positive_negative_lookahead OK")
