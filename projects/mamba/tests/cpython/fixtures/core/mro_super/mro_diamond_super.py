# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# MRO + super() — #2783.
#
# Covers Python's C3-linearized method resolution order (MRO) and
# cooperative `super()` calls in diamond inheritance.
#
# Diamond:
#
#       A
#      / \
#     B   C
#      \ /
#       D
#
# Expected C3 MRO for D: [D, B, C, A, object]. Each class's __init__
# / method invokes super(), so a single call on D chains through
# every class in MRO order exactly once.
#
# Clauses:
#   1. D.__mro__ matches the C3 linearization.
#   2. Cooperative super() chain — calling `D().method()` runs every
#      class's method in MRO order, once each.
#   3. super().__init__() propagates through the diamond — `A` runs
#      exactly once (the "diamond problem" Python solves with C3).
#   4. super() with explicit type+obj arguments dispatches the same
#      as bare super().
#   5. Skipped middle — if B.method does NOT call super(), the chain
#      stops at B; C and A are NOT reached.
#   6. Inconsistent base order raises TypeError at class-creation
#      time (C3 cannot linearize). `class X(B, C)` and `class Y(C, B)`
#      together would be inconsistent for a deeper diamond; we use
#      the classic L-shape failure to trigger TypeError.
#
# Every print line tagged `[mro-super]` so failure output names
# MRO semantics.


TRACE = []


class A:
    def __init__(self):
        TRACE.append("A.__init__")

    def method(self):
        TRACE.append("A.method")
        return ["A"]


class B(A):
    def __init__(self):
        TRACE.append("B.__init__")
        super().__init__()

    def method(self):
        TRACE.append("B.method")
        result = super().method()
        return ["B"] + result


class C(A):
    def __init__(self):
        TRACE.append("C.__init__")
        super().__init__()

    def method(self):
        TRACE.append("C.method")
        result = super().method()
        return ["C"] + result


class D(B, C):
    def __init__(self):
        TRACE.append("D.__init__")
        super().__init__()

    def method(self):
        TRACE.append("D.method")
        result = super().method()
        return ["D"] + result


# Clause 1: MRO follows C3 linearization.
mro_names = [cls.__name__ for cls in D.__mro__]
print("[mro-super] clause-1 mro:", mro_names)


# Clause 2: cooperative super() chain through `method`.
TRACE.clear()
d = D()
# Snapshot __init__ trace before calling `method`.
init_trace = TRACE[:]
print("[mro-super] clause-2 init-trace:", init_trace)

TRACE.clear()
chain = d.method()
print("[mro-super] clause-2 method-trace:", TRACE[:])
print("[mro-super] clause-2 chain:", chain)


# Clause 3: A.__init__ ran exactly once during D() construction
# (the diamond problem — naive multiple inheritance would call A
# twice).
print("[mro-super] clause-3 A-init-count:", init_trace.count("A.__init__"))


# Clause 4: super() with explicit type + obj args dispatches the
# same as bare super().
class E(D):
    def method(self):  # pyright: ignore[reportIncompatibleMethodOverride]
        TRACE.append("E.method")
        # Both forms must yield the same dispatch.
        result_bare = super().method()
        result_explicit = super(E, self).method()  # noqa: UP008
        return result_bare, result_explicit


TRACE.clear()
e = E()
bare, explicit = e.method()
print("[mro-super] clause-4 bare-equals-explicit:", bare == explicit)
print("[mro-super] clause-4 chain:", bare)


# Clause 5: skipped middle — B' below doesn't call super().method,
# so C.method and A.method are NOT reached from D2.method.
class B2(A):
    def method(self):
        TRACE.append("B2.method")
        # NO super() call — chain stops here.
        return ["B2-stop"]


class D2(B2, C):
    def method(self):
        TRACE.append("D2.method")
        return ["D2"] + super().method()


TRACE.clear()
d2 = D2()  # __init__ chain ignored for this clause
TRACE.clear()
print("[mro-super] clause-5 short-chain:", d2.method())
print("[mro-super] clause-5 trace:", TRACE[:])


# Clause 6: C3 inconsistency raises TypeError at class-creation time.
# Using siblings B and C (both inherit from A): X forces B-before-C,
# Y forces C-before-B; Z(X, Y) cannot linearize.
class X(B, C):
    pass


class Y(C, B):
    pass


try:
    class Z(X, Y):  # pyright: ignore[reportGeneralTypeIssues] — intentional MRO failure
        pass

    print("[mro-super] clause-6 typeerror: <unexpected-no-error>")
except TypeError as exc:
    print("[mro-super] clause-6 typeerror:", type(exc).__name__)
