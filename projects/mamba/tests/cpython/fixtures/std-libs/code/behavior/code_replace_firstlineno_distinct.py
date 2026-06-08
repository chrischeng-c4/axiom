# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_replace_firstlineno_distinct"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: replacing co_firstlineno shifts the line number and yields a distinct, unequal code object"""
import types

c1 = (lambda: 1).__code__
c_shift = c1.replace(co_firstlineno=c1.co_firstlineno + 5)
assert c_shift.co_firstlineno == c1.co_firstlineno + 5, "firstlineno shifted by 5"
assert c1 != c_shift, "firstlineno change -> unequal code object"

print("code_replace_firstlineno_distinct OK")
