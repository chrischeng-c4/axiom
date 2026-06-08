# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_replace_preserves_unchanged"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: co.replace(co_name=same) returns a new code object preserving the unchanged field (co_name stays equal)"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
nc = co.replace(co_name="sample")
assert nc is not co, "replace returns a new code object"
assert nc.co_name == "sample", f"replaced co_name = {nc.co_name!r}"
assert nc.co_argcount == co.co_argcount, "untouched field preserved"

print("code_replace_preserves_unchanged OK")
