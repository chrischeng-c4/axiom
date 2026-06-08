# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""__new__ construction protocol and object.__new__/__init__ rules."""


# __new__ may return an unrelated object; __init__ is then skipped.
class StrMaker:
    def __new__(cls, arg):
        if isinstance(arg, str):
            return [1, 2, 3]
        return object.__new__(cls)

    def __init__(self, arg):
        self.arg = arg


assert StrMaker("hi") == [1, 2, 3]      # returned a list, not an instance
made = StrMaker(7)
assert isinstance(made, StrMaker)
assert made.arg == 7                    # __init__ ran for the real instance


# __new__ may return an instance of a sibling/subclass; that type's
# __init__ runs.
class Base:
    def __new__(cls, arg):
        if isinstance(arg, int):
            return object.__new__(Sub)
        return object.__new__(cls)


class Sub(Base):
    def __init__(self, arg):
        self.foo = arg


d = Base(5)
assert isinstance(d, Sub)
assert d.foo == 5


# staticmethod __new__ receives the class as its first positional arg.
class StaticNew:
    @staticmethod
    def __new__(*args):
        return args


assert StaticNew(1, 2) == (StaticNew, 1, 2)


class StaticNewChild(StaticNew):
    pass


assert StaticNewChild(1, 2) == (StaticNewChild, 1, 2)


# object.__new__ accepts extra args only when __init__ is overridden;
# object.__init__ accepts extra args only when __new__ is overridden.
class HasInit:
    def __init__(self, foo):
        self.foo = foo


object.__new__(HasInit)             # ok, no extra args
object.__new__(HasInit, 5)          # ok: __init__ is overridden

try:
    object.__init__(HasInit(3), 5)  # not ok: __new__ is the default one
    print("init_extra: no_raise")
except TypeError:
    print("init_extra: TypeError")

print("new_protocol OK")
