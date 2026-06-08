# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_matmul_operator_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not operator-transparent: proxy @ diverges from CPython (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy forwards @, reflected @, and @= to __matmul__/__rmatmul__/__imatmul__"""
import weakref


# Matrix-multiply forwarding (@, reflected, in-place).
class Matmul:
    def __matmul__(self, other):
        return 1729

    def __rmatmul__(self, other):
        return -163

    def __imatmul__(self, other):
        return 561


mm = Matmul()
p_mm = weakref.proxy(mm)
assert p_mm @ 5 == 1729, "proxy @"
assert 5 @ p_mm == -163, "reflected @"
p_mm @= 5
assert p_mm == 561, f"proxy @= -> {p_mm!r}"

print("proxy_matmul_operator_forwarding OK")
