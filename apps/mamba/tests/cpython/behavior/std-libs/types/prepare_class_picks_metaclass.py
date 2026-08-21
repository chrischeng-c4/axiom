# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "prepare_class_picks_metaclass"
# subject = "types.prepare_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.prepare_class: prepare_class selects the most-derived metaclass and returns the namespace it provides via __prepare__"""
import types

expected_ns = {}


class A(type):
    def __new__(*args, **kwargs):
        return type.__new__(*args, **kwargs)

    def __prepare__(*args):
        return expected_ns


with_a = types.new_class("WithA", (object,), {"metaclass": A})
meta, namespace, kwds = types.prepare_class("Derived", (with_a,),
                                            {"metaclass": type})
assert meta is A
assert namespace is expected_ns
assert len(kwds) == 0

print("prepare_class_picks_metaclass OK")
