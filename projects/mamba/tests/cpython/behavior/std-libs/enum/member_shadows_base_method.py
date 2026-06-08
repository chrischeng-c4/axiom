# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "member_shadows_base_method"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: a subclass member named like a base-Enum method shadows the method: the name resolves to the member (an instance of the subclass) with its value"""
import enum


class Base(enum.Enum):
    def test(self):
        return "method"


class Sub(Base):
    test = 1


# The member shadows the inherited method: Sub.test is a member, not a method.
assert type(Sub.test) is Sub, f"shadowed member type = {type(Sub.test)!r}"
assert Sub.test.value == 1, "shadowing member keeps its value"

print("member_shadows_base_method OK")
