# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "abstract_method_super_default_body"
# subject = "abc.abstractmethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.abstractmethod: an abstractmethod may carry a default body reachable via super() from the concrete override"""
import abc


class DefaultImpl(abc.ABC):
    @abc.abstractmethod
    def compute(self, x: int) -> int:
        return x * 10  # default body, callable via super()


class UseDefault(DefaultImpl):
    def compute(self, x: int) -> int:
        return super().compute(x) + 1


result = UseDefault().compute(5)
assert result == 51, f"super().compute default body chained: {result!r}"

print("abstract_method_super_default_body OK")
