# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/mro_super: language-area behavior asserts (CPython 3.12 oracle)."""

# Generic language behavior checks.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert isinstance(True, int)
assert isinstance(1, int)
assert type(()) is tuple
assert type([]) is list
assert type({}) is dict
assert len("abc") == 3
assert list(range(3)) == [0, 1, 2]


# The implicit `__class__` cell is bound inside any method body — including
# class/static methods — to the enclosing class object.
class X:
    @classmethod
    def via_classmethod(cls):
        return __class__

    @staticmethod
    def via_staticmethod():
        return __class__

    def via_instancemethod(self):
        return __class__


assert X.via_classmethod() is X
assert X.via_staticmethod() is X
assert X().via_instancemethod() is X


# A bare `super()` proxy reports its own type as `super`, and a `super`
# subclass instance reports the subclass.
class ReportsClass:
    def bare(self):
        return super().__class__

    def subclassed(self):
        return MySuper(ReportsClass, self).__class__


class MySuper(super):
    pass


assert ReportsClass().bare() is super
assert ReportsClass().subclassed() is MySuper


# `super` resolves as an ordinary global name: a local class named `super`
# shadows the builtin, so `super()` refers to the shadow, not the proxy.
def shadowed():
    class super:  # noqa: A001 — intentional shadow of builtin
        msg = "quite super"

    class C:
        def method(self):
            return super().msg

    return C().method()


assert shadowed() == "quite super"

print("behavior OK")
