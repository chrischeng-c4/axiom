# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "utility_functions_callable"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: the class-creation utilities new_class / prepare_class / resolve_bases are all callable"""
import types

assert callable(types.new_class)
assert callable(types.prepare_class)
assert callable(types.resolve_bases)

print("utility_functions_callable OK")
