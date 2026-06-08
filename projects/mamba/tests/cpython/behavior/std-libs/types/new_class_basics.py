# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "new_class_basics"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: new_class('C') builds a fresh class deriving from object, and explicit empty bases/kwds/exec_body default to the same result"""
import types

# new_class('C') builds a fresh class deriving from object.
C = types.new_class("C")
assert C.__name__ == "C"
assert C.__bases__ == (object,)

# Explicit empty bases/kwds/exec-body default to the same result.
D = types.new_class("D", (), {}, None)
assert D.__name__ == "D"
assert D.__bases__ == (object,)

print("new_class_basics OK")
