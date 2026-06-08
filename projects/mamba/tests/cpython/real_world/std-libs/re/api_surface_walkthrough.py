# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "real_world"
# case = "api_surface_walkthrough"
# subject = "re"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re: a downstream consumer drives match/search/findall/fullmatch/sub/subn/split/compile/escape together over realistic inputs, asserting each result and a deterministic aggregate over a multi-group bulk findall"""
import re

# 1. match — anchored at start.
m = re.match(r"(\w+)\s+(\d+)", "hello 42 world")
assert m is not None, "match returned None"
assert m.group(1) == "hello" and m.group(2) == "42", f"match groups = {m.groups()!r}"
assert m.group(0) == "hello 42", f"match group(0) = {m.group(0)!r}"
assert re.match(r"\d+", "abc 123") is None, "match should be None when BOS doesn't match"

# 2. search — first hit anywhere.
s = re.search(r"(\d+)", "abc 123 def 456")
assert s is not None and s.group(1) == "123", f"search group(1) = {s.group(1) if s else None!r}"

# 3. findall — single-group bulk scan.
hits = re.findall(r"(\w+)=(\d+)", "x=1 y=22 z=333")
assert hits == [("x", "1"), ("y", "22"), ("z", "333")], f"findall hits = {hits!r}"

# 4. Multi-group findall at scale; aggregate to a deterministic scalar.
parts = []
for i in range(1000):
    parts.append("%d.%d.%d.%d " % (i % 250, (i // 4) % 250, (i // 16) % 250, i % 7))
corpus = "".join(parts)
multi = re.findall(r"(\d+)\.(\d+)\.(\d+)\.(\d+)", corpus)
assert len(multi) == 1000, f"multi-group findall len = {len(multi)}"
assert multi[0] == ("0", "0", "0", "0"), f"multi[0] = {multi[0]!r}"
total = 0
for tup in multi:
    total += int(tup[0]) + int(tup[1]) + int(tup[2]) + int(tup[3])
assert total > 0, "aggregate over captured groups is non-trivial"

# 5. fullmatch — whole-string schema check.
fm = re.fullmatch(r"(\w+)-(\d+)", "alpha-99")
assert fm is not None and fm.group(1) == "alpha" and fm.group(2) == "99", "fullmatch groups"
assert re.fullmatch(r"(\w+)-(\d+)", "alpha-99 trailing") is None, "fullmatch rejects trailing chars"

# 6. sub — literal-string replacement.
assert re.sub(r"\d+", "N", "a1 b22 c333") == "aN bN cN", "sub literal"

# 7. subn — substitution + count.
result, count = re.subn(r"\d+", "X", "a1 b22 c333")
assert result == "aX bX cX" and count == 3, f"subn = {(result, count)!r}"

# 8. split — pattern-driven splitter.
assert re.split(r"\s+", "foo bar  baz") == ["foo", "bar", "baz"], "split"

# 9. compile — Pattern reused for findall and match.
pat = re.compile(r"(\w+):(\d+)")
assert pat.findall("k1:10 k2:20 k3:30") == [("k1", "10"), ("k2", "20"), ("k3", "30")], "compiled findall"
mp = pat.match("token:777 rest")
assert mp is not None and mp.group(1) == "token" and mp.group(2) == "777", "compiled match groups"

# 10. escape — metachar-safe literal builder; round-trips with search.
esc = re.escape("a.b*c")
assert re.search(esc, "before a.b*c after") is not None, "escape+search round-trip"
assert re.search(re.escape("plain"), "see plain text") is not None, "escape identity on letters"

# 11. groups() — full tuple.
g = re.match(r"(\w+)\s+(\w+)\s+(\d+)", "alice bob 42")
assert g is not None and g.groups() == ("alice", "bob", "42"), f"groups() = {g.groups() if g else None!r}"

print("api_surface_walkthrough OK")
