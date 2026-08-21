"""Behavior contract for language walrus operator (:=).

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: := assigns and returns the value
_x = (_y := 7)
assert _y == 7, f"y = {_y!r}"
assert _x == 7, f"x = {_x!r}"

# Rule 2: Walrus in while reads from enclosing scope after loop
_nums = [3, 1, 4, 1, 5]
_idx = 0
_total = 0
while (_idx < len(_nums)) and (_v := _nums[_idx]):
    _total += _v
    _idx += 1
assert _total == 14, f"while total = {_total!r}"

# Rule 3: Walrus assigns to enclosing (not comprehension) scope —
# _matched takes the value of the LAST element visited by the filter
_matched = None
_items = [0, 0, 42, 0]
_found = [_n for _n in _items if (_matched := _n) > 0]
assert _found == [42], f"found = {_found!r}"
# Last element tested by the filter is 0, so _matched ends up 0
assert _matched == 0, f"matched = {_matched!r}"

# Rule 4: Walrus preserves normal short-circuit for and/or
_calls = []
def _track(label: str, val: bool) -> bool:
    _calls.append(label)
    return val

# Short-circuit: second not called if first is False
_calls.clear()
_r = (_track("A", False)) and (_b2 := _track("B", True))
assert _calls == ["A"], f"short-circuit calls = {_calls!r}"

# Rule 5: Walrus in if-elif chains
_val2 = 15
if (_sq := _val2 ** 2) < 100:
    _bucket = "small"
elif _sq < 400:
    _bucket = "medium"
else:
    _bucket = "large"
assert _sq == 225, f"sq = {_sq!r}"
assert _bucket == "medium", f"bucket = {_bucket!r}"

# Rule 6: Walrus inside function argument (assigns in function's scope)
def _double(n: int) -> int:
    return n * 2

_saved = 0
_res = _double(_saved := 8)
assert _saved == 8, f"saved = {_saved!r}"
assert _res == 16, f"res = {_res!r}"

# Rule 7: Repeated walrus overwrites
_ = (_w := 1)
_ = (_w := 2)
_ = (_w := 3)
assert _w == 3, f"overwrite = {_w!r}"

# Rule 8: Walrus in list comprehension filter — result type
_data = list(range(10))
_doubled = [_d for _n in _data if (_d := _n * 3) > 10]
assert _doubled == [12, 15, 18, 21, 24, 27], f"doubled = {_doubled!r}"

print("behavior OK")
