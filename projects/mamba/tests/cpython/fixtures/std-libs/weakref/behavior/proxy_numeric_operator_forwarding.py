# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_numeric_operator_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not operator-transparent: proxy + float diverges from CPython (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy forwards +, reflected +, // and //= to the referent's numeric dunders"""
import weakref


# Numeric forwarding through a float subclass referent.
class MyFloat(float):
    pass


num = MyFloat(2.0)
p_num = weakref.proxy(num)
assert p_num + 1.0 == 3.0, "proxy + float"
assert 1.0 + p_num == 3.0, "float + proxy (reflected)"


# Floor-division, both normal and in-place.
class Divver:
    def __floordiv__(self, other):
        return 42

    def __ifloordiv__(self, other):
        return 21


div = Divver()
p_div = weakref.proxy(div)
assert p_div // 5 == 42, "proxy //"
p_div //= 5
assert p_div == 21, f"proxy //= -> {p_div!r}"

print("proxy_numeric_operator_forwarding OK")
