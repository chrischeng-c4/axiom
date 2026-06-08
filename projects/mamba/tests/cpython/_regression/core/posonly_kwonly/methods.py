# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Positional-only / keyword-only markers inside methods.
#
# Methods are ordinary functions whose first parameter is `self`; the
# `/` and `*` markers apply after `self`, and `self` itself is bound by
# the descriptor protocol on a bound call but supplied explicitly on an
# unbound (class-attribute) call.
#
# Clauses:
#   1. Bound + unbound calls of a posonly method behave identically.
#   2. Passing a posonly method parameter by keyword raises TypeError.
#   3. Bound + unbound calls of a kwonly method behave identically.
#   4. Calling an unbound kwonly method without `self` raises TypeError.
#   5. Dunder-prefixed parameter names are mangled to the *local* name
#      inside the body, so `__a` is still readable as the parameter.
#
# Each print line is tagged `[posonly-methods]`.


class PosOnly:
    def f(self, a, b, /):
        return (a, b)


inst = PosOnly()
print("[posonly-methods] clause-1 bound:", inst.f(1, 2))
print("[posonly-methods] clause-1 unbound:", PosOnly.f(inst, 1, 2))

try:
    inst.f(1, b=2)  # type: ignore[call-arg]
    print("[posonly-methods] clause-2 posonly-as-kw: <unexpected-no-error>")
except TypeError:
    print("[posonly-methods] clause-2 posonly-as-kw: TypeError")


class KwOnly:
    def f(self, *, k1=1, k2=2):
        return (k1, k2)


kinst = KwOnly()
print("[posonly-methods] clause-3 bound:", kinst.f(k1=10, k2=20))
print("[posonly-methods] clause-3 unbound:", KwOnly.f(kinst, k1=10, k2=20))

try:
    KwOnly.f(k1=1, k2=2)  # type: ignore[call-arg]  # missing self
    print("[posonly-methods] clause-4 missing-self: <unexpected-no-error>")
except TypeError:
    print("[posonly-methods] clause-4 missing-self: TypeError")


class Mangled:
    # `__a` is name-mangled inside the class body, but a posonly /
    # kwonly parameter named `__a` is still bindable to its default and
    # readable in the body under the mangled local name.
    def posonly(self, __a=42, /):
        return __a

    def kwonly(self, *, __a=7):
        return __a


m = Mangled()
print("[posonly-methods] clause-5 posonly-default:", m.posonly())
print("[posonly-methods] clause-5 kwonly-default:", m.kwonly())
