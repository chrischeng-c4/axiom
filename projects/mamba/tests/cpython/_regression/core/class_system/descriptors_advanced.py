# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Data descriptors, __set_name__, and bound-method binding (CPython 3.12)."""


# A data descriptor (defines __get__ and __set__) takes precedence over the
# instance __dict__.
class Logged:
    def __set_name__(self, owner, name):
        self.storage = "_" + name

    def __get__(self, obj, objtype=None):
        if obj is None:
            return self
        return getattr(obj, self.storage, None)

    def __set__(self, obj, value):
        setattr(obj, self.storage, value * 2)


class Model:
    field = Logged()


m = Model()
m.field = 5
assert m.field == 10                 # __set__ doubled, __get__ returned it
assert m._field == 10                # stored under the __set_name__ name
assert isinstance(Model.field, Logged)   # class access returns descriptor


# __set_name__ is called once per descriptor with its owner and attr name.
class Named:
    def __set_name__(self, owner, name):
        self.name = name

    def __get__(self, obj, objtype=None):
        return self.name


class Holder:
    a = Named()
    b = Named()


assert Holder.a == "a"
assert Holder.b == "b"


# Non-data descriptor (only __get__) is shadowed by an instance attribute.
class NonData:
    def __get__(self, obj, objtype=None):
        return "descriptor"


class Mixed:
    val = NonData()


x = Mixed()
assert x.val == "descriptor"
x.__dict__["val"] = "instance"
assert x.val == "instance"           # instance dict wins for non-data descr


# Plain function is a descriptor: accessing it on an instance binds self.
class WithMethod:
    def f(self):
        return 7


inst = WithMethod()
assert type(WithMethod.f).__name__ == "function"
assert type(inst.f).__name__ == "method"
assert inst.f.__func__ is WithMethod.f
assert inst.f() == 7
assert WithMethod.f(inst) == 7       # unbound call with explicit self

print("descriptors_advanced OK")
