# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isbuiltin_true_for_len"
# subject = "inspect.isbuiltin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isbuiltin: isbuiltin is True for a builtin like len"""
import inspect

assert inspect.isbuiltin(len), "isbuiltin(len)"

print("isbuiltin_true_for_len OK")
