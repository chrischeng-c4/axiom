# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "override_sets_runtime_flag"
# subject = "typing.override"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.override: @override is a pass-through that sets the __override__ flag at runtime: a Child.run decorated with @override returns 2, Child.run.__override__ is True, and the undecorated Base.run has no __override__ attribute"""
from typing import override


# @override marks a method with the __override__ flag at runtime (a no-op pass-through).
class Base:
    def run(self) -> int:
        return 1


class Child(Base):
    @override
    def run(self) -> int:
        return 2


assert Child().run() == 2
assert Child.run.__override__ is True
assert not hasattr(Base.run, "__override__")

print("override_sets_runtime_flag OK")
