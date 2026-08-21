# Operational AssertionPass seed for the `pprint` module. No prior
# pprint fixture lives in pass/. Surface: `pprint.pformat(obj)`
# returns a `str` whose body contains the canonical text rendering
# of dict keys, list elements, and tuple members. `pprint.pformat`
# accepts `indent=` and `width=` kwargs without raising. The
# in-place `pprint.pprint(obj)` returns `None` (its side-effect is
# printing). `pprint.isreadable` returns a bool.
import pprint
_ledger: list[int] = []

# pformat returns a str
s_dict = pprint.pformat({"a": 1, "b": 2})
assert isinstance(s_dict, str); _ledger.append(1)
assert "a" in s_dict; _ledger.append(1)
assert "1" in s_dict; _ledger.append(1)
assert "b" in s_dict; _ledger.append(1)
assert "2" in s_dict; _ledger.append(1)

# pformat on list
s_list = pprint.pformat([10, 20, 30])
assert isinstance(s_list, str); _ledger.append(1)
assert "10" in s_list; _ledger.append(1)
assert "20" in s_list; _ledger.append(1)
assert "30" in s_list; _ledger.append(1)

# pformat on tuple
s_tup = pprint.pformat((42, 99))
assert isinstance(s_tup, str); _ledger.append(1)
assert "42" in s_tup; _ledger.append(1)
assert "99" in s_tup; _ledger.append(1)

# indent= kwarg accepted
s_indent = pprint.pformat([1, 2, 3], indent=4)
assert isinstance(s_indent, str); _ledger.append(1)

# width= kwarg accepted
s_width = pprint.pformat({"a": 1, "b": 2, "c": 3}, width=10)
assert isinstance(s_width, str); _ledger.append(1)

# pprint side-effect: returns None
assert pprint.pprint("x") is None; _ledger.append(1)
assert pprint.pprint([1, 2]) is None; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pprint_pformat_ops {sum(_ledger)} asserts")
