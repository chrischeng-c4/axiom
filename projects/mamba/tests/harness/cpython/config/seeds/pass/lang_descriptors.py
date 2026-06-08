# lang_descriptors.py — #3359 axis-1 descriptor protocol seed.
#
# Exercises:
#   1. Data descriptor (__get__ + __set__) — read-back of stored value
#   2. Data descriptor write — value propagates through __set__
#   3. Non-data descriptor (__get__ only) returning a constant
#   4. @property as a data descriptor (get + set via @setter)
#   5. @classmethod descriptor binding cls
#   6. @staticmethod descriptor (no binding)
#   7. Descriptor __get__ receives owner class on instance access
#
# Mamba quirks (tracked separately):
#   * Class-access to a descriptor (e.g., `C.x`) returns the descriptor
#     object itself instead of invoking __get__(None, C). Intentionally
#     NOT exercised — instance-access path covered.
#   * instance.__dict__ returns None inside __get__/__set__ (#3503).
#
# Contract: AssertionError → Fail; MAMBA_ASSERTION_PASS → AssertionPass.

_ledger: list[int] = []

# (1) Data descriptor with __get__ and __set__
class _Logged:
    def __init__(self):
        self._value = 0
    def __get__(self, obj, owner=None):
        return self._value
    def __set__(self, obj, value):
        self._value = value

class _C:
    x = _Logged()

_c = _C()
assert _c.x - 0 == 0, f"data descriptor initial 0, got {_c.x!r}"
_ledger.append(1)

# (2) Data descriptor write goes through __set__
_c.x = 42
assert _c.x - 42 == 0, f"data descriptor after set 42, got {_c.x!r}"
_ledger.append(1)

_c.x = 99
assert _c.x - 99 == 0, f"data descriptor after second set 99, got {_c.x!r}"
_ledger.append(1)

# (3) Non-data descriptor (__get__ only) returns constant
class _Getter:
    def __get__(self, obj, owner=None):
        return 77

class _D:
    y = _Getter()

_d = _D()
assert _d.y - 77 == 0, f"non-data descriptor returns 77, got {_d.y!r}"
_ledger.append(1)

# (4) @property — built-in data descriptor
class _E:
    def __init__(self):
        self._z = 10
    @property
    def z(self):
        return self._z
    @z.setter
    def z(self, value):
        self._z = value

_e = _E()
assert _e.z - 10 == 0, f"@property get 10, got {_e.z!r}"
_ledger.append(1)

_e.z = 100
assert _e.z - 100 == 0, f"@property setter set 100, got {_e.z!r}"
_ledger.append(1)

# (5) @classmethod descriptor binds cls
class _M:
    @classmethod
    def cm(cls, x):
        return (cls.__name__, x)
    @staticmethod
    def sm(x):
        return ("static", x)

assert _M.cm(1) == ("_M", 1), f"classmethod via class, got {_M.cm(1)!r}"
_ledger.append(1)

_m_inst = _M()
assert _m_inst.cm(3) == ("_M", 3), (
    f"classmethod via instance, got {_m_inst.cm(3)!r}"
)
_ledger.append(1)

# (6) @staticmethod descriptor — no binding
assert _M.sm(2) == ("static", 2), f"staticmethod via class, got {_M.sm(2)!r}"
_ledger.append(1)

assert _m_inst.sm(4) == ("static", 4), (
    f"staticmethod via instance, got {_m_inst.sm(4)!r}"
)
_ledger.append(1)

# (7) __get__ receives owner class on instance access
class _TypeGetter:
    def __get__(self, obj, owner=None):
        return owner.__name__ if owner is not None else "no-owner"

class _N:
    x = _TypeGetter()

_n = _N()
assert _n.x == "_N", f"descriptor sees owner='_N' on inst access, got {_n.x!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_descriptors {sum(_ledger)} asserts")
