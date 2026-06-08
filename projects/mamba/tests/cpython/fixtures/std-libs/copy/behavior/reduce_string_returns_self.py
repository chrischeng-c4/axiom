# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "reduce_string_returns_self"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: a __reduce__ returning a string means both copy and deepcopy return the object itself"""
import copy


class GlobalRef:
    def __reduce__(self):
        return ""  # the global-reference pickle convention -> return self


g = GlobalRef()
assert copy.copy(g) is g, "reduce string result: copy returns self"
assert copy.deepcopy(g) is g, "reduce string result: deepcopy returns self"

print("reduce_string_returns_self OK")
