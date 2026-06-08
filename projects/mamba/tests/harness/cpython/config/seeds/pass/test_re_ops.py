# Operational AssertionPass seed for the `re` stdlib module.
# Surface: match groups, findall, sub, split, search, compile,
# common char classes (\d, \w, \s).
# Companion to stub/test_re.py — vendored unittest seed.
import re
_ledger: list[int] = []
m = re.match(r"(\d+)-(\w+)", "42-hello")
assert m.group(0) == "42-hello"; _ledger.append(1)
assert m.group(1) == "42"; _ledger.append(1)
assert m.group(2) == "hello"; _ledger.append(1)
assert re.findall(r"\d+", "a1 b22 c333") == ["1", "22", "333"]; _ledger.append(1)
assert re.sub(r"\s+", "_", "hello  world\tfoo") == "hello_world_foo"; _ledger.append(1)
assert re.split(r"[,;]", "a,b;c,d") == ["a", "b", "c", "d"]; _ledger.append(1)
assert re.match(r"\d+", "abc") is None; _ledger.append(1)
s = re.search(r"\d+", "abc123def")
assert s is not None; _ledger.append(1)
assert s.group(0) == "123"; _ledger.append(1)
pat = re.compile(r"[A-Z]+")
assert pat.findall("Hello World ABC") == ["H", "W", "ABC"]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_re_ops {sum(_ledger)} asserts")
