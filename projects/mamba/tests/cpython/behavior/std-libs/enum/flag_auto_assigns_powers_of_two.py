# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "flag_auto_assigns_powers_of_two"
# subject = "enum.Flag"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Flag: auto() in a Flag body assigns successive powers of two (1, 2, 4) and iteration yields only the canonical single-bit members"""
import enum


class Color(enum.Flag):
    RED = enum.auto()
    BLUE = enum.auto()
    GREEN = enum.auto()


assert Color.RED.value == 1, f"RED = {Color.RED.value!r}"
assert Color.BLUE.value == 2, f"BLUE = {Color.BLUE.value!r}"
assert Color.GREEN.value == 4, f"GREEN = {Color.GREEN.value!r}"
assert list(Color) == [Color.RED, Color.BLUE, Color.GREEN], "iter is canonical members"

print("flag_auto_assigns_powers_of_two OK")
