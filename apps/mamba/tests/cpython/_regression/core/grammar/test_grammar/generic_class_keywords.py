# RUN: parse

class Meta(type):
    pass

class MyClass[T](metaclass=Meta):
    pass

class Concrete[T: int](list[T], metaclass=Meta):
    pass
