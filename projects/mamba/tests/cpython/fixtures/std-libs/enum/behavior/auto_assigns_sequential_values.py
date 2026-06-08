# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "auto_assigns_sequential_values"
# subject = "enum.auto"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.auto: auto() in an Enum body assigns 1, 2, 3, ... in definition order"""
import enum


class Fruit(enum.Enum):
    APPLE = enum.auto()
    BANANA = enum.auto()
    CHERRY = enum.auto()


assert Fruit.APPLE.value == 1, f"auto 1 = {Fruit.APPLE.value!r}"
assert Fruit.BANANA.value == 2, f"auto 2 = {Fruit.BANANA.value!r}"
assert Fruit.CHERRY.value == 3, f"auto 3 = {Fruit.CHERRY.value!r}"

print("auto_assigns_sequential_values OK")
