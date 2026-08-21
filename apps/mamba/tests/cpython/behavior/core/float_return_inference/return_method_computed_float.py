# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_method_computed_float"
# subject = "method returning a computed float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A method returning a computed float must yield the correct float value."""


class Box:
    def __init__(self, value):
        self.value = value

    def half(self):
        return self.value / 2


b = Box(9)
r = b.half()
assert r == 4.5, r
assert isinstance(r, float), type(r)
assert r * 2 == 9.0, r
print("return_method_computed_float OK")
