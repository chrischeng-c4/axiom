# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "walrus_in_boolean_short_circuit"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus on the right of `or` binds and short-circuits correctly: False or (z := fn(7)) is 7, z == 7, and fn was called exactly once"""
# A walrus on the right of `or` binds and short-circuits correctly.
calls: list = []
def fn(v: int) -> int:
    calls.append(v)
    return v

result = False or (z := fn(7))
assert result == 7, f"or walrus = {result!r}"
assert z == 7, f"z = {z!r}"
assert calls == [7], f"calls = {calls!r}"

print("walrus_in_boolean_short_circuit OK")
