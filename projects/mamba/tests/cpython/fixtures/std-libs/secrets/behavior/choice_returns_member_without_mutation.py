# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "choice_returns_member_without_mutation"
# subject = "secrets.choice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.choice: choice(seq) returns an element of seq across repeated draws and never mutates the input sequence"""
import secrets

_items = list(range(10))
_orig = list(_items)
for _draw in range(20):
    _c = secrets.choice(_items)
    assert _c in _items, f"choice not in sequence: {_c}"
assert _items == _orig, "choice must not modify the input sequence"

print("choice_returns_member_without_mutation OK")
