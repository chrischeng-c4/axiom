# RUN: parse
# Extracted from CPython Lib/test/test_opcache.py — syntax constructs only.


# Descriptor protocol
class Descriptor:
    def __get__(self, instance, owner):
        return lambda: 1


class Base:
    d = Descriptor()


class Derived(Base):
    def f(self):
        return super().d()


# Descriptor added after class creation
class LazyDescriptor:
    pass


class WithLazyDescriptor:
    def __init__(self):
        self.x = 1
    x = LazyDescriptor()


def read_attr(o):
    return o.x


# Metaclass with descriptor
class MetaDescriptor:
    pass


class Metaclass(type):
    attribute = MetaDescriptor()


class MetaClass(metaclass=Metaclass):
    attribute = True


# Metaclass with @property
class PropertyMeta(type):
    @property
    def attribute(self):
        return True


class PropertyMetaClass(metaclass=PropertyMeta):
    attribute = False


# Metaclass set descriptor after class creation
class SetterMeta(type):
    pass


class SetterMetaClass(metaclass=SetterMeta):
    attribute = True


@property
def dynamic_attribute(self):
    return False


# Metaclass __getattribute__ override
class GetAttrMeta(type):
    def __getattribute__(self, name):
        return True


class GetAttrClass(metaclass=GetAttrMeta):
    attribute = False


# Metaclass swap via __class__
class OldMetaclass(type):
    @property
    def attribute(self):
        return True


class NewMetaclass(type):
    @property
    def attribute(self):
        return False


class SwappableClass(metaclass=OldMetaclass):
    pass


# __slots__ usage
class SlottedBase:
    __slots__ = ("slot",)


class SneakySlot:
    __slots__ = ("shadowed",)
    shadowing = SlottedBase.slot


class BorrowedSlot:
    borrowed = SlottedBase.slot


# Method cache patterns
class MethodDescriptor:
    pass


class MethodHost:
    attribute = MethodDescriptor()


def method_reader():
    instance = MethodHost()
    return instance.attribute


# Metaclass descriptor shadows class method
class MethodPropertyMeta(type):
    @property
    def attribute(self):
        return lambda: True


class MethodMetaClass(metaclass=MethodPropertyMeta):
    def attribute():
        return False


# Type descriptor shadows
class TypeShadow:
    mro = None


class TypeBaseShadow:
    __base__ = None


class TypeNameShadow:
    __name__ = "Spam"


# Function defaults modification
def defaults_0():
    pass


def defaults_1(x):
    pass


def defaults_2(x, y):
    pass


defaults_0.__defaults__ = (None,)
defaults_1.__defaults__ = (None, None)
defaults_2.__defaults__ = (None, None, None)

for _ in range(1025):
    defaults_0()
    defaults_1(None)
    defaults_1()
    defaults_2(None, None)
    defaults_2(None)
    defaults_2()
