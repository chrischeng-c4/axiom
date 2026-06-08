# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_immutable_attribute_assignment"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter instances are immutable: assigning an attribute raises AttributeError"""
import inspect

P = inspect.Parameter

imm = P("spam", kind=P.KEYWORD_ONLY)
_raised = False
try:
    imm.foo = "bar"
except AttributeError:
    _raised = True
assert _raised, "expected AttributeError on attribute set"

print("parameter_immutable_attribute_assignment OK")
