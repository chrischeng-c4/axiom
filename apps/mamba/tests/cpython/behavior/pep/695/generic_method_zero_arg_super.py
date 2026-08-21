# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "generic_method_zero_arg_super"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "zero-arg super() inside a def greet[T] generic method diverges on mamba (PEP 695 method type-param path; probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a generic method def greet[T](self, tag) can still use zero-arg super() to reach its base: Sub().greet(1) == 'parent-sub'"""


# A generic method can still use zero-arg super() to reach its base.
class Parent:
    def greet(self):
        return "parent"


class Sub(Parent):
    def greet[T](self, tag: int) -> str:
        return super().greet() + "-sub"


assert Sub().greet(1) == "parent-sub"

print("generic_method_zero_arg_super OK")
