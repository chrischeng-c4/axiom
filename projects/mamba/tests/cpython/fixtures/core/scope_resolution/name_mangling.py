# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Private name mangling — scope/name-resolution rule.
#
# Any identifier of the form `__name` (two leading underscores, at most
# one trailing underscore) that textually appears inside a class body is
# rewritten to `_ClassName__name`, using the name of the *immediately
# enclosing* class. This is a purely lexical, compile-time transform.
#
# Clauses:
#   1. An attribute `self.__x` set in a class is stored under
#      `_Class__x`; the plain `__x` attribute does not exist.
#   2. Mangling uses the immediately enclosing class. A `__arg`
#      parameter of a method on an inner class is `_Inner__arg`, not
#      the outer class's mangling, so calling it with the outer-class
#      keyword raises TypeError.
#   3. Dunder names (leading AND trailing `__`) are NOT mangled.
#   4. A single leading underscore is NOT mangled.
#   5. A class-body variable `__secret` is mangled to `_Class__secret`.

# Clause 1: attribute mangling.
class Holder:
    def __init__(self):
        self.__x = 5

    def get(self):
        return self.__x


h = Holder()
assert h.get() == 5
assert h._Holder__x == 5
assert not hasattr(h, "__x")
print("[mangle] clause-1 attr:", h.get(), h._Holder__x)


# Clause 2: mangling uses the immediately enclosing class.
class Outer:
    def make(self):
        __arg = 1  # local; mangling here would be _Outer__arg

        class Inner:
            def g(self, __arg):  # parameter mangles to _Inner__arg
                return __arg

        # Correct keyword uses the INNER class's mangling.
        return Inner().g(_Inner__arg=2)


assert Outer().make() == 2

# The outer-class keyword does NOT match -> unexpected keyword.
class O2:
    def make(self):
        class D:
            def g(self, __arg):
                return __arg

        return D().g


closure = O2().make()
try:
    closure(_O2__arg=2)
    raise AssertionError("expected TypeError")
except TypeError:
    print("[mangle] clause-2 wrong-class-kw: TypeError")


# Clause 3: dunder names are not mangled.
class WithDunder:
    def __init__(self):
        self.__ready__ = True


assert WithDunder().__ready__ is True
print("[mangle] clause-3 dunder-not-mangled: ok")


# Clause 4: single leading underscore is not mangled.
class Single:
    def __init__(self):
        self._y = 7


assert Single()._y == 7
print("[mangle] clause-4 single-underscore: ok")


# Clause 5: class-body variable mangling.
class Vars:
    __secret = "hidden"


assert Vars._Vars__secret == "hidden"
print("[mangle] clause-5 classvar:", Vars._Vars__secret)

print("name_mangling OK")
