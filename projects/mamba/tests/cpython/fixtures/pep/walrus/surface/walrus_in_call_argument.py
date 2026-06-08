# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "surface"
# case = "walrus_in_call_argument"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus used as a function-call argument binds the name in the surrounding scope and passes the value to the callee"""
# := as a call argument binds in the surrounding scope and passes the value.
def double(n: int) -> int:
    return n * 2

saved = None
res = double(saved := 5)
assert saved == 5, f"saved = {saved!r}"
assert res == 10, f"double = {res!r}"

print("walrus_in_call_argument OK")
