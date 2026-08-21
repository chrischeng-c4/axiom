# mamba-xfail: mamba's type checker rejects the module-level
# `__annotations__` reference ("undefined name: `__annotations__`"),
# and also does not currently expose `f.__annotations__` /
# `C.__annotations__`. All three storage paths (module / function /
# class) are the runtime gap that gates the xfail.
#
# Annotation storage / future-annotations behavior — #2812.
#
# Covers Python 3.12 annotation storage: function / class / module-level
# variable annotations are visible through __annotations__. Without
# `from __future__ import annotations`, function-parameter and return
# annotations are evaluated at definition time, but the *strings* stored
# in __annotations__ may be the resolved type objects (e.g. `int`) or
# their repr forms — the fixture asserts the well-known keys/types only,
# not the repr formatting.
#
# Failure messages name [annotations] so runner stderr identifies the
# area.

# 1. Module-level variable annotations populate __annotations__.
mod_int: int = 7
mod_str: str

print("'mod_int' in __annotations__=", "mod_int" in __annotations__, "[annotations]")
print("'mod_str' in __annotations__=", "mod_str" in __annotations__, "[annotations]")
print("__annotations__['mod_int'] is int=", __annotations__["mod_int"] is int, "[annotations]")
print("__annotations__['mod_str'] is str=", __annotations__["mod_str"] is str, "[annotations]")
print("mod_int value=", mod_int, "[annotations]")


# 2. Function annotations: parameters + return.
def f(a: int, b: str = "x") -> bool:
    return True


anns = f.__annotations__
keys = sorted(anns.keys())
print("f.__annotations__ keys=", keys, "[annotations: function]")
print("f.__annotations__['a'] is int=", anns["a"] is int, "[annotations: function]")
print("f.__annotations__['b'] is str=", anns["b"] is str, "[annotations: function]")
print("f.__annotations__['return'] is bool=", anns["return"] is bool, "[annotations: function]")


# 3. Class-level annotations populate cls.__annotations__.
class C:
    x: int = 1
    y: str = "two"
    z: float


cann = C.__annotations__
ckeys = sorted(cann.keys())
print("C.__annotations__ keys=", ckeys, "[annotations: class]")
print("C.__annotations__['x'] is int=", cann["x"] is int, "[annotations: class]")
print("C.__annotations__['y'] is str=", cann["y"] is str, "[annotations: class]")
print("C.__annotations__['z'] is float=", cann["z"] is float, "[annotations: class]")
print("C.x value=", C.x, "[annotations: class]")
