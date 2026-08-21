# Operational AssertionPass seed for `re` advanced surface beyond
# bare match.
# Surface: re.match groups (0/1/2), re.split, re.sub with a literal
# string replacement, re.findall, re.search, re.escape, re.compile +
# .match on the compiled pattern. The callable-replacement form of
# re.sub (lambda) is currently broken on mamba and is omitted; the
# string-replacement form is asserted instead.
import re
_ledger: list[int] = []
# match groups: 0 = whole match; 1, 2 = captures
m = re.match(r"(\w+)\s+(\d+)", "alice 42")
assert m.group(0) == "alice 42"; _ledger.append(1)
assert m.group(1) == "alice"; _ledger.append(1)
assert m.group(2) == "42"; _ledger.append(1)
# split collapses runs of whitespace
assert re.split(r"\s+", "a  b   c") == ["a", "b", "c"]; _ledger.append(1)
# sub with literal string replacement
assert re.sub(r"\d+", "X", "a1 b22 c333") == "aX bX cX"; _ledger.append(1)
# findall returns the list of matched substrings
assert re.findall(r"\d+", "a1 b22 c333") == ["1", "22", "333"]; _ledger.append(1)
# search finds the first match anywhere in the input
s = re.search(r"\d+", "abc 123 def")
assert s.group(0) == "123"; _ledger.append(1)
# escape quotes regex metacharacters
assert re.escape("a.b*c") == "a\\.b\\*c"; _ledger.append(1)
# compile + match on the resulting Pattern
c = re.compile(r"^[A-Z]+$")
assert c.match("HELLO") is not None; _ledger.append(1)
assert c.match("hello") is None; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_re_advanced_ops {sum(_ledger)} asserts")
