# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "behavior"
# case = "matmul_dispatches_to_dunder"
# subject = "operator.matmul"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.matmul: matmul dispatches to a type's __matmul__ dunder"""
import operator

class Mat:
    def __matmul__(self, other):
        return other - 1


assert operator.matmul(Mat(), 42) == 41, "matmul via __matmul__"

print("matmul_dispatches_to_dunder OK")
