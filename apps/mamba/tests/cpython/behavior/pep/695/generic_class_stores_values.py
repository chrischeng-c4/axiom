# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "generic_class_stores_values"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a generic class Box[T] instantiates and stores any value regardless of the declared T: Box(42).value==42 and Box('hi').value=='hi'"""


# A generic class instantiates and stores values regardless of the declared T.
class Box[T]:
    def __init__(self, value: T) -> None:
        self.value = value


assert Box(42).value == 42
assert Box("hi").value == "hi"

print("generic_class_stores_values OK")
