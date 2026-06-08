"""Behavior contract for language metaclasses.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: metaclass __new__ is called at class definition time
_calls: list = []

class _ObserveMeta(type):
    def __new__(mcs, name, bases, namespace):
        _calls.append(f"new:{name}")
        return super().__new__(mcs, name, bases, namespace)
    def __init__(cls, name, bases, namespace):
        _calls.append(f"init:{name}")
        super().__init__(name, bases, namespace)

_calls.clear()
class _MyClass(metaclass=_ObserveMeta):
    pass

assert "new:_MyClass" in _calls, f"calls = {_calls!r}"
assert "init:_MyClass" in _calls, f"calls = {_calls!r}"
assert _calls.index("new:_MyClass") < _calls.index("init:_MyClass"), "__new__ before __init__"

# Rule 2: Subclasses inherit metaclass
class _Child(_MyClass):
    pass

assert type(_Child) is _ObserveMeta, f"child metaclass = {type(_Child)!r}"

# Rule 3: Metaclass can add methods to all subclasses
class _AutoRepr(type):
    def __new__(mcs, name, bases, namespace):
        def __repr__(self) -> str:
            return f"<{type(self).__name__}>"
        namespace.setdefault("__repr__", __repr__)
        return super().__new__(mcs, name, bases, namespace)

class _Widget(metaclass=_AutoRepr):
    pass

_w = _Widget()
assert repr(_w) == "<_Widget>", f"auto repr = {repr(_w)!r}"

# Rule 4: type() 3-arg form creates class with metaclass=type
_Klass = type("_Klass", (object,), {"value": 99})
assert _Klass.value == 99, f"Klass.value = {_Klass.value!r}"  # type: ignore[attr-defined]
assert type(_Klass) is type, f"type(_Klass) = {type(_Klass)!r}"

# Rule 5: __prepare__ lets metaclass control the namespace (e.g., OrderedDict)
from collections import OrderedDict

class _OrderedMeta(type):
    @classmethod
    def __prepare__(mcs, name, bases, **kwargs):
        return OrderedDict()

    def __new__(mcs, name, bases, namespace):
        return super().__new__(mcs, name, bases, dict(namespace))

class _Ordered(metaclass=_OrderedMeta):
    first = 1
    second = 2
    third = 3

assert _Ordered.first == 1, f"first = {_Ordered.first!r}"
assert _Ordered.third == 3, f"third = {_Ordered.third!r}"

# Rule 6: ABCMeta enforces abstractmethod
from abc import ABCMeta, abstractmethod

class _Animal(metaclass=ABCMeta):
    @abstractmethod
    def speak(self) -> str: ...

class _Dog(_Animal):
    def speak(self) -> str:
        return "woof"

assert _Dog().speak() == "woof", f"speak = {_Dog().speak()!r}"

_raised = False
try:
    _Animal()  # type: ignore[abstract]
except TypeError:
    _raised = True
assert _raised, "abstract instantiation should raise TypeError"

# Rule 7: isinstance with metaclass-created classes
_Dyn2 = type("_Dyn2", (object,), {})
_obj = _Dyn2()
assert isinstance(_obj, _Dyn2), "isinstance with dynamic class"
assert isinstance(_obj, object), "dynamic class is subclass of object"

print("behavior OK")
