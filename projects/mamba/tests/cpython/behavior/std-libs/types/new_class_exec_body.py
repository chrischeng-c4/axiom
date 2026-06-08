# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "new_class_exec_body"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: an exec_body callback populates the new class namespace before the class is created"""
import types


def body(ns):
    ns["value"] = 42


E = types.new_class("E", (), {}, body)
assert E.value == 42

print("new_class_exec_body OK")
