# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "init_unpacks_tuple_value"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a tuple member value with a custom __init__ unpacks fields onto each member, the original tuple stays as .value, and an @enum.property computes a derived attribute"""
import enum


class Planet(enum.Enum):
    EARTH = (5.976e24, 6.378140e6)
    MARS = (6.421e23, 3.397200e6)

    def __init__(self, mass, radius):
        self.mass = mass
        self.radius = radius

    @enum.property
    def gravity(self):
        return 6.673e-11 * self.mass / (self.radius * self.radius)


assert Planet.EARTH.value == (5.976e24, 6.378140e6), "tuple value retained"
assert Planet.MARS.mass == 6.421e23, "field unpacked by __init__"
assert round(Planet.EARTH.gravity, 2) == 9.8, f"derived gravity = {Planet.EARTH.gravity!r}"

print("init_unpacks_tuple_value OK")
