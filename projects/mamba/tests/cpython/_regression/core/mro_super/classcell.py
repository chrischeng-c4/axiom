# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/mro_super: __classcell__ propagation and __new__ reassignment.

When a class body uses zero-arg `super()` or `__class__`, the compiler
emits a `__classcell__` entry in the class namespace. A custom metaclass
must propagate that cell to `type.__new__`, which fills it with the new
class object. Mishandling the cell is a hard error.
"""


# A metaclass that forwards the namespace unchanged propagates __classcell__
# correctly, so the implicit __class__ cell is bound and super() works.
class GoodMeta(type):
    def __new__(mcls, name, bases, namespace):
        return super().__new__(mcls, name, bases, namespace)


class WithClassRef(metaclass=GoodMeta):
    def f(self):
        return __class__


assert WithClassRef().f() is WithClassRef
print("[classcell] good-meta:", WithClassRef().f().__name__)


# Dropping __classcell__ before type.__new__ leaves the implicit cell empty;
# CPython raises RuntimeError when the class body referenced __class__.
class DropMeta(type):
    def __new__(mcls, name, bases, namespace):
        namespace.pop("__classcell__", None)
        return super().__new__(mcls, name, bases, namespace)


try:

    class Dropped(metaclass=DropMeta):
        def f(self):
            return __class__

    print("[classcell] dropped: no_raise")
except RuntimeError as e:
    assert "__class__ not set" in str(e)
    print("[classcell] dropped:", type(e).__name__)


# Building a *different* class from the same namespace consumes the cell with
# the wrong class object; CPython rejects the mismatch with TypeError.
class WrongCellMeta(type):
    def __new__(mcls, name, bases, namespace):
        real = super().__new__(mcls, name, bases, namespace)
        type("Decoy", (), namespace)  # fills the cell with a foreign class
        return real


try:

    class WrongCell(metaclass=WrongCellMeta):
        def f(self):
            return __class__

    print("[classcell] wrong-cell: no_raise")
except TypeError as e:
    assert "__class__ set to" in str(e)
    print("[classcell] wrong-cell:", type(e).__name__)


# Reassigning __new__ via __init_subclass__ must not break the super().__new__
# chain in a grandchild class.
class Base:
    def __new__(cls):
        return object.__new__(cls)

    def __init_subclass__(cls):
        if "__new__" not in cls.__dict__:
            cls.__new__ = cls.__new__


class Mid(Base):
    pass


class Leaf(Mid):
    def __new__(cls):
        return super().__new__(cls)


assert type(Leaf()) is Leaf
print("[classcell] reassigned-new:", type(Leaf()).__name__)
