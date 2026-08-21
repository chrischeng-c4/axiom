# Property descriptor — getter / setter / deleter — #2784.
#
# Covers Python's `property` descriptor protocol: getter, setter,
# deleter, plus the AttributeError that surfaces when a property
# exposes a getter but no setter or deleter.
#
# Clauses:
#   1. Getter dispatch — reading the attribute runs the fget.
#   2. Setter dispatch — assignment runs the fset (and validates
#      input in user code).
#   3. Deleter dispatch — `del obj.attr` runs the fdel.
#   4. AttributeError on assignment when property has no setter
#      ("read-only property").
#   5. AttributeError on `del` when property has no deleter.
#   6. property without a getter (only setter) raises AttributeError
#      when the attribute is read.
#   7. Decorator chain `@x.setter` / `@x.deleter` returns the new
#      property; the original `x` still has only the getter (proves
#      .setter/.deleter return NEW property objects, not mutating
#      the original).
#
# Every print line tagged `[property]` so failure output names
# property semantics.


class Temperature:
    def __init__(self, celsius=0):
        self._c = celsius
        self.deleted = False

    @property
    def celsius(self):
        return self._c

    @celsius.setter
    def celsius(self, value):
        if value < -273.15:
            raise ValueError("below absolute zero")
        self._c = value

    @celsius.deleter
    def celsius(self):
        self.deleted = True
        self._c = 0


t = Temperature(20)

# Clause 1: getter dispatch.
print("[property] clause-1 get:", t.celsius)


# Clause 2: setter dispatch + getter sees the new value.
t.celsius = 100
print("[property] clause-2 after-set:", t.celsius)

# Setter input validation.
try:
    t.celsius = -1000
    print("[property] clause-2 validation: <unexpected-no-error>")
except ValueError as exc:
    print("[property] clause-2 validation:", type(exc).__name__)
# Value unchanged after the failed assignment.
print("[property] clause-2 after-validation-fail:", t.celsius)


# Clause 3: deleter dispatch.
del t.celsius
print("[property] clause-3 after-delete-flag:", t.deleted)
print("[property] clause-3 after-delete-value:", t.celsius)


# Clause 4: read-only property — assignment raises AttributeError.
class ReadOnly:
    @property
    def x(self):
        return 42


ro = ReadOnly()
print("[property] clause-4 readonly-get:", ro.x)
try:
    ro.x = 99  # pyright: ignore[reportAttributeAccessIssue] — intentional read-only test
    print("[property] clause-4 attrerror: <unexpected-no-error>")
except AttributeError as exc:
    print("[property] clause-4 attrerror:", type(exc).__name__)


# Clause 5: no-deleter property — `del` raises AttributeError.
class NoDelete:
    def __init__(self):
        self._v = 1

    @property
    def v(self):
        return self._v

    @v.setter
    def v(self, value):
        self._v = value


nd = NoDelete()
nd.v = 7
print("[property] clause-5 setter-works:", nd.v)
try:
    del nd.v  # pyright: ignore[reportAttributeAccessIssue] — intentional no-deleter test
    print("[property] clause-5 attrerror: <unexpected-no-error>")
except AttributeError as exc:
    print("[property] clause-5 attrerror:", type(exc).__name__)


# Clause 6: setter-only property — reading raises AttributeError.
class WriteOnly:
    def __init__(self):
        self._stash = []

    # property(fget=None, fset=...) makes the attribute write-only.
    def _set(self, value):
        self._stash.append(value)

    write = property(fset=_set)


wo = WriteOnly()
wo.write = "hello"
print("[property] clause-6 stash:", wo._stash)
try:
    _ = wo.write
    print("[property] clause-6 attrerror: <unexpected-no-error>")
except AttributeError as exc:
    print("[property] clause-6 attrerror:", type(exc).__name__)


# Clause 7: @x.setter and @x.deleter return NEW property objects.
class Chained:
    @property
    def x(self):
        return 1


orig = Chained.x


class Chained2(Chained):
    @Chained.x.setter
    def x(self, value):
        self._stored = value


# Chained.x.setter returns a new property; the original Chained.x
# property object is unchanged.
print("[property] clause-7 orig-fset-is-none:", orig.fset is None)
print("[property] clause-7 child-fset-is-some:", Chained2.x.fset is not None)
print("[property] clause-7 orig-fget-shared:", Chained2.x.fget is orig.fget)
