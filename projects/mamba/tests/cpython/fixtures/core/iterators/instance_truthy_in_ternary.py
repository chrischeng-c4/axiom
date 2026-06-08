# Guard for the ternary Instance-truthiness fix (4316aa8c). `x if obj else y`
# used to always pick `else` because the Branch terminator tested the raw
# NaN-boxed pointer's low bits instead of calling mb_is_truthy.

class Obj:
    pass

class Empty:
    def __bool__(self):
        return False

class Full:
    def __bool__(self):
        return True

class LenZero:
    def __len__(self):
        return 0

class LenNonZero:
    def __len__(self):
        return 3

o = Obj()
print("no dunder:", "truthy" if o else "falsy")
print("__bool__ False:", "truthy" if Empty() else "falsy")
print("__bool__ True:", "truthy" if Full() else "falsy")
print("__len__ 0:", "truthy" if LenZero() else "falsy")
print("__len__ nonzero:", "truthy" if LenNonZero() else "falsy")

# and/or short-circuit still works
a = o and "picked"
print("and:", a)
b = None or "fallback"
print("or:", b)
