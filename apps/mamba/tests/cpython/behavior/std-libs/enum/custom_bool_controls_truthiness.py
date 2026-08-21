# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "custom_bool_controls_truthiness"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a custom __bool__ controls member truthiness; without one, every Enum member is truthy regardless of value"""
import enum


class RealLogic(enum.Enum):
    TRUE = True
    FALSE = False

    def __bool__(self):
        return bool(self._value_)


assert RealLogic.TRUE, "custom __bool__ makes TRUE truthy"
assert not RealLogic.FALSE, "custom __bool__ makes FALSE falsy"


# Without __bool__, every Enum member is truthy regardless of value.
class PlainLogic(enum.Enum):
    TRUE = True
    FALSE = False


assert PlainLogic.FALSE, "plain Enum member is always truthy"

print("custom_bool_controls_truthiness OK")
