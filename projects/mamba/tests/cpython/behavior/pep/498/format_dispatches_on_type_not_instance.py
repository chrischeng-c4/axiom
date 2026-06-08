# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "format_dispatches_on_type_not_instance"
# subject = "fstring.evaluation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.evaluation: f-string format dispatches __format__ on the type, ignoring a per-instance __format__ attribute, while calling the bound attribute directly uses it"""
import types

class Fmt:
    def __format__(self, spec):
        return "class"

obj = Fmt()
obj.__format__ = types.MethodType(lambda self, spec: "instance", obj)
assert f"{obj}" == "class"
assert format(obj) == "class"
# But calling the bound instance attribute directly does use it.
assert obj.__format__("") == "instance"

print("format_dispatches_on_type_not_instance OK")
