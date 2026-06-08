# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "class_type_params_and_parameters"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "cls.__type_params__ / __parameters__ return None on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: nested generic classes each keep __type_params__, and a generic class also gains a matching __parameters__ tuple from Generic"""


# Nested generic classes each keep their own params; static methods see all four.
class Outer[A, B]:
    class Inner[C, D]:
        @staticmethod
        def get():
            return (A, B, C, D)


oa, ob, oc, od = Outer.Inner.get()
assert Outer.__type_params__ == (oa, ob)
assert Outer.Inner.__type_params__ == (oc, od)
# Generic classes also gain __parameters__ (from Generic) matching the params.
assert Outer.__parameters__ == (oa, ob)
assert Outer.Inner.__parameters__ == (oc, od)

print("class_type_params_and_parameters OK")
