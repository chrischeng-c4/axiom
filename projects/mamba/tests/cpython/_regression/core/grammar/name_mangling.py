# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/grammar: private name mangling inside class bodies (CPython 3.12 oracle).

Distilled from CPython TestSpecifics::test_mangling. A leading double
underscore (without a trailing one) is textually rewritten to
``_ClassName__name`` everywhere it appears inside the class body.
"""


class Counter:
    def __init__(self):
        self.__value = 0          # stored as _Counter__value

    def bump(self):
        self.__value += 1
        return self.__value


c = Counter()
assert c.bump() == 1
assert c.bump() == 2
# The mangled attribute is what actually lands on the instance.
assert c._Counter__value == 2
assert not hasattr(c, "__value")
print("attribute_mangling: ok")


class Names:
    def f(self):
        __mangled = 1          # local var name is mangled too
        __not_mangled__ = 2    # dunder (trailing __) is NOT mangled
        return (__mangled, __not_mangled__)


varnames = Names.f.__code__.co_varnames
assert "_Names__mangled" in varnames
assert "__not_mangled__" in varnames
assert "__mangled" not in varnames
print("local_mangling: ok")


# Mangling depends on the lexically enclosing class, so two classes mangle
# the same source name to different attributes.
class A:
    def __init__(self):
        self.__x = "a"


class B:
    def __init__(self):
        self.__x = "b"


assert A()._A__x == "a"
assert B()._B__x == "b"
print("name_mangling OK")
