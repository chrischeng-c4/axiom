"""Behavior contract for language descriptors.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: Data descriptor overrides instance __dict__
class _Override:
    def __set_name__(self, owner, name: str):
        self._name = "_" + name
    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        return obj.__dict__.get(self._name, 0)
    def __set__(self, obj, value: int):
        obj.__dict__[self._name] = value * 2  # always doubles

class _Holder:
    val = _Override()

_h = _Holder()
_h.val = 5
assert _h.val == 10, f"data desc override = {_h.val!r}"

# Rule 2: Non-data descriptor doesn't override instance __dict__
class _NonData:
    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        return "from_descriptor"

class _Owner:
    attr = _NonData()

_o = _Owner()
assert _o.attr == "from_descriptor", f"non-data desc = {_o.attr!r}"
# Instance __dict__ entry beats non-data descriptor
_o.__dict__["attr"] = "from_instance"
assert _o.attr == "from_instance", f"instance beats non-data = {_o.attr!r}"

# Rule 3: __get__ called with obj=None when accessed on class
class _ClassCheck:
    def __get__(self, obj, objtype=None):
        if obj is None:
            return "class-access"
        return "instance-access"

class _Target:
    desc = _ClassCheck()

assert _Target.desc == "class-access", f"class access = {_Target.desc!r}"
assert _Target().desc == "instance-access", f"instance access = {_Target().desc!r}"

# Rule 4: __delete__ called on del
class _Deletable:
    def __set_name__(self, owner, name: str):
        self._name = "_" + name
        self._log = []
    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        return obj.__dict__.get(self._name)
    def __set__(self, obj, value):
        obj.__dict__[self._name] = value
    def __delete__(self, obj):
        self._log.append("deleted")
        obj.__dict__.pop(self._name, None)

class _WithDel:
    item = _Deletable()

_wd = _WithDel()
_wd.item = 99
del _wd.item
assert _WithDel.item._log == ["deleted"], f"delete log = {_WithDel.item._log!r}"
assert _wd.item is None, f"after del = {_wd.item!r}"

# Rule 5: property getter/setter/deleter
class _Bounded:
    def __init__(self):
        self._v = 0

    @property
    def v(self) -> int:
        return self._v

    @v.setter
    def v(self, val: int) -> None:
        self._v = max(0, min(100, val))

    @v.deleter
    def v(self) -> None:
        self._v = 0

_b = _Bounded()
_b.v = 50
assert _b.v == 50, f"v=50 = {_b.v!r}"
_b.v = 200
assert _b.v == 100, f"v=200 clamped = {_b.v!r}"
_b.v = -5
assert _b.v == 0, f"v=-5 clamped = {_b.v!r}"
del _b.v
assert _b.v == 0, f"after del = {_b.v!r}"

# Rule 6: Descriptor protocol inheritance
class _Base:
    def __get__(self, obj, objtype=None):
        return 42

class _Child(_Base):
    pass  # inherits __get__

class _UseChild:
    attr = _Child()

assert _UseChild().attr == 42, f"inherited desc = {_UseChild().attr!r}"

print("behavior OK")
