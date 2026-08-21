# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Decorators stacked with the descriptor protocol (CPython data model).
# Decorators often return descriptors (objects with __get__) and get stacked
# with the builtin classmethod. This pins down how a custom descriptor and
# classmethod compose in either order, plus classmethod.__get__ binding rules.

from types import MethodType


# Custom classmethod-like descriptor: __get__ binds self to the owner class via
# MethodType so the call passes the class as the first argument.
class MyClassMethod:
    def __init__(self, func):
        if isinstance(func, classmethod):
            func = func.__func__   # unwrap a stacked builtin classmethod
        self.func = func

    def __call__(self, cls):
        return self.func(cls)

    def __get__(self, instance, owner=None):
        if owner is None:
            owner = type(instance)
        return MethodType(self, owner)


class A:
    @MyClassMethod
    def plain(cls):
        return cls

    @classmethod
    @MyClassMethod
    def cm_outer(cls):           # builtin classmethod wraps custom one
        return cls

    @MyClassMethod
    @classmethod
    def cm_inner(cls):           # custom unwraps the builtin classmethod
        return cls


# All three resolve `cls` to A whether accessed via the class or an instance.
a = A()
for obj in (A, a):
    assert obj.plain() is A
    assert obj.cm_outer() is A
    assert obj.cm_inner() is A


# classmethod.__get__ binding rules, made explicit. The owner (second arg, or
# type(instance) when omitted) is what gets bound as `cls`.
def ident(cls):
    return cls


assert classmethod(ident).__get__(a)() is A         # owner inferred from a
assert classmethod(ident).__get__(a, A)() is A      # explicit owner A
assert classmethod(ident).__get__(A, A)() is A      # instance is the class
assert classmethod(ident).__get__(A)() is type(A)   # owner inferred -> type(A)

# Our custom descriptor follows the same binding contract.
assert MyClassMethod(ident).__get__(a)() is A
assert MyClassMethod(ident).__get__(A)() is type(A)


# A plain decorator returning a descriptor composes over an inner classmethod:
# its __get__ delegates to the already-bound classmethod and forwards the call.
class BoundWrapper:
    def __init__(self, wrapped):
        self.__wrapped__ = wrapped

    def __call__(self, *args, **kwargs):
        return self.__wrapped__(*args, **kwargs)


class Wrapper:
    def __init__(self, wrapped):
        self.__wrapped__ = wrapped

    def __get__(self, instance, owner):
        return BoundWrapper(self.__wrapped__.__get__(instance, owner))


def wrap(wrapped):
    return Wrapper(wrapped)


class Class:
    @wrap
    @classmethod
    def inner(cls):
        return "spam"


assert Class.inner() == "spam"
assert Class().inner() == "spam"

print("descriptor_interplay OK")
