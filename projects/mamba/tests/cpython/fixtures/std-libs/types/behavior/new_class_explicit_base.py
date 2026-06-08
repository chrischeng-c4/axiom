# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "new_class_explicit_base"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: new_class with an explicit base honors that base in __bases__ and inherits its attributes"""
import types


class Base:
    tag = "base"


F = types.new_class("F", (Base,))
assert F.__bases__ == (Base,)
assert F.tag == "base"

print("new_class_explicit_base OK")
