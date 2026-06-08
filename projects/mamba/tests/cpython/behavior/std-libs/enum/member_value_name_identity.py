# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "member_value_name_identity"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: an Enum member exposes .value and .name, is an instance of its class, and equals itself by identity and ==; distinct members are !="""
import enum


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


assert Color.RED.value == 1, f"RED.value = {Color.RED.value!r}"
assert Color.GREEN.name == "GREEN", f"GREEN.name = {Color.GREEN.name!r}"
assert isinstance(Color.BLUE, Color), "BLUE is an instance of Color"

assert Color.RED is Color.RED, "identity"
assert Color.RED == Color.RED, "equality"
assert Color.RED != Color.GREEN, "distinct members are not equal"

print("member_value_name_identity OK")
