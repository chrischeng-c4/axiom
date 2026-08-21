# RUN: parse
# Complex class definition syntax fixture (#574)

# --- simple class ---
class Empty:
    pass

# --- class with body ---
class WithBody:
    x = 1
    y = "hello"

# --- class with methods ---
class WithMethods:
    def __init__(self):
        self.x = 0

    def method(self):
        return self.x

    @staticmethod
    def static_method():
        return 42

    @classmethod
    def class_method(cls):
        return cls()

# --- single inheritance ---
class Child(Parent):
    pass

# --- multiple inheritance ---
class MultiChild(Parent1, Parent2, Parent3):
    pass

# --- keyword arguments in class ---
class Meta(metaclass=type):
    pass

class WithKeywords(Base, metaclass=type, extra=True):
    pass

# --- class with decorators ---
@decorator
class Decorated:
    pass

@decorator1
@decorator2
@decorator3
class MultiDecorated:
    pass

@decorator_with_args(1, 2, key="value")
class DecoratedWithArgs:
    pass

# --- class with type annotations ---
class Annotated:
    # NOTE: bare annotation without value not supported
    name: str = ""
    age: int = 0
    items: list[int] = []

# --- class with __slots__ ---
class Slotted:
    __slots__ = ("x", "y", "z")

# --- class with properties ---
class WithProperties:
    def __init__(self):
        self._value = 0

    @property
    def value(self):
        return self._value

    @value.setter
    def value(self, val):
        self._value = val

    @value.deleter
    def value(self):
        del self._value

# --- nested classes ---
class Outer:
    class Inner:
        class InnerInner:
            pass

# --- class with all dunder methods ---
class FullDunder:
    def __init__(self):
        pass
    def __repr__(self):
        return "FullDunder()"
    def __str__(self):
        return "full"
    def __eq__(self, other):
        return True
    def __hash__(self):
        return 0
    def __len__(self):
        return 0
    def __getitem__(self, key):
        pass
    def __setitem__(self, key, value):
        pass
    def __delitem__(self, key):
        pass
    def __contains__(self, item):
        return False
    def __iter__(self):
        return iter([])
    def __next__(self):
        raise StopIteration
    def __enter__(self):
        return self
    def __exit__(self, *args):
        pass
    def __call__(self, *args, **kwargs):
        pass
    def __add__(self, other):
        return self
    def __sub__(self, other):
        return self
    def __mul__(self, other):
        return self
    def __lt__(self, other):
        return False
    def __le__(self, other):
        return False
    def __gt__(self, other):
        return False
    def __ge__(self, other):
        return False
    def __bool__(self):
        return True

# --- abstract class pattern ---
class AbstractBase:
    def method(self):
        raise NotImplementedError

    def abstract_method(self):
        ...

# --- mixin pattern ---
class SerializerMixin:
    def serialize(self):
        return str(self)

class LoggerMixin:
    def log(self, msg):
        print(msg)

class Combined(SerializerMixin, LoggerMixin):
    pass

# --- class with complex methods ---
class Complex:
    def method_with_defaults(self, a, b=10, c="hello"):
        pass

    def method_with_star(self, a, *, key):
        pass

    def method_with_args_kwargs(self, *args, **kwargs):
        pass

    def method_with_all(self, a, b=1, *args, key=True, **kwargs):
        pass

# --- dataclass-like pattern ---
class Point:
    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y

    def __repr__(self):
        return f"Point({self.x}, {self.y})"

    def __eq__(self, other):
        return self.x == other.x and self.y == other.y

# --- enum-like pattern ---
class Color:
    RED = 1
    GREEN = 2
    BLUE = 3

# --- singleton pattern ---
class Singleton:
    _instance = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

# --- class with class variables and instance variables ---
class MixedVars:
    class_var = "shared"
    class_list: list = []

    def __init__(self):
        self.instance_var = "unique"

# --- PEP 695 generic class ---
class Stack[T]:
    def __init__(self):
        self.items = []  # NOTE: attribute type annotation not supported

    def push(self, item: T):
        self.items.append(item)

    def pop(self) -> T:
        return self.items.pop()

class Pair[T, U]:
    def __init__(self, first: T, second: U):
        self.first = first
        self.second = second

# --- class body with diverse statements ---
class DiverseBody:
    # assignments
    x = 1
    y: int = 2

    # conditional
    if True:
        z = 3
    else:
        z = 4

    # loop
    items = []
    for i in range(5):
        items.append(i)

    # function def
    def method(self):
        pass

    # nested class
    class Nested:
        pass

    # try/except
    try:
        import json
    except ImportError:
        json = None

    # delete
    del x
