# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_equality_on_replace"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: code equality: c.replace() with no changes compares equal to c, while c.replace(co_name=other) compares unequal"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
assert co.replace() == co, "no-op replace() compares equal"
assert co.replace(co_name="renamed") != co, "renaming makes the code object unequal"

print("code_equality_on_replace OK")
