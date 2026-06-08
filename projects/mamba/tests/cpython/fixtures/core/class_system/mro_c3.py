# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""C3 linearization: MRO ordering and inconsistent-MRO errors (CPython 3.12)."""


# A complex multiple-inheritance lattice produces a single deterministic MRO.
class Boat:
    pass


class DayBoat(Boat):
    pass


class WheelBoat(Boat):
    pass


class EngineLess(DayBoat):
    pass


class SmallMultihull(DayBoat):
    pass


class PedalWheelBoat(EngineLess, WheelBoat):
    pass


class SmallCatamaran(SmallMultihull):
    pass


class Pedalo(PedalWheelBoat, SmallCatamaran):
    pass


def names(cls):
    return [c.__name__ for c in cls.__mro__]


assert names(PedalWheelBoat) == [
    "PedalWheelBoat", "EngineLess", "DayBoat", "WheelBoat", "Boat", "object"
]
assert names(Pedalo) == [
    "Pedalo", "PedalWheelBoat", "EngineLess", "SmallCatamaran",
    "SmallMultihull", "DayBoat", "WheelBoat", "Boat", "object"
]

# Each base appears exactly once and a class precedes all of its bases.
mro = Pedalo.__mro__
assert len(set(mro)) == len(mro)
assert mro.index(PedalWheelBoat) < mro.index(EngineLess)
assert mro.index(EngineLess) < mro.index(DayBoat)


# An inconsistent base ordering cannot be linearized -> TypeError.
class A:
    pass


class B(A):
    pass


try:
    type("X", (A, B), {})       # A before its own subclass B
    print("inconsistent: no_raise")
except TypeError as e:
    print("inconsistent: TypeError", str(e)[:40])

# Mutually incompatible diamonds also fail.
class HVGrid(WheelBoat, DayBoat):
    pass


class VHGrid(DayBoat, WheelBoat):
    pass


try:
    type("Confused", (HVGrid, VHGrid), {})
    print("confused: no_raise")
except TypeError:
    print("confused: TypeError")

print("mro_c3 OK")
