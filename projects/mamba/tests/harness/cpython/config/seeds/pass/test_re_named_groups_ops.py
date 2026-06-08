# Operational AssertionPass seed for re named groups and fullmatch
# surfaces beyond test_re_ops / test_re_advanced_ops.
# Surface: (?P<name>pat) named-group capture, .group('name') and
# .groupdict() access, re.fullmatch success and failure, findall over
# multi-group regex returning tuples, re.split over the whole input,
# bare numeric .group(N) on a named-group regex.
import re
_ledger: list[int] = []

# (?P<name>...) captures the matched text under the name
m = re.match(r"(?P<num>\d+)-(?P<word>\w+)", "42-hello")
assert m is not None; _ledger.append(1)
assert m.group("num") == "42"; _ledger.append(1)
assert m.group("word") == "hello"; _ledger.append(1)
# groupdict materializes ALL named groups as a name → value mapping
assert m.groupdict() == {"num": "42", "word": "hello"}; _ledger.append(1)
# Numeric .group(N) on a named-group regex still works (1-indexed)
assert m.group(1) == "42"; _ledger.append(1)
assert m.group(2) == "hello"; _ledger.append(1)
# .groups() returns the tuple of all capturing groups
assert m.groups() == ("42", "hello"); _ledger.append(1)

# fullmatch — pattern must match the ENTIRE input string
fm = re.fullmatch(r"\d+", "123")
assert fm is not None; _ledger.append(1)
# Partial match: fullmatch FAILS where match() would have succeeded
fm_partial = re.fullmatch(r"\d+", "123abc")
assert fm_partial is None; _ledger.append(1)

# findall over a multi-group regex returns a list of tuples
parts = re.findall(r"(\d+)-(\w+)", "1-a, 2-b, 3-c")
assert parts == [("1", "a"), ("2", "b"), ("3", "c")]; _ledger.append(1)

# split treats the regex as the delimiter
assert re.split(r",\s*", "a, b,c,  d") == ["a", "b", "c", "d"]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_re_named_groups_ops {sum(_ledger)} asserts")
