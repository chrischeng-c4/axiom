# RUN: parse
# CPython 3.12 test_grammar: decorator syntax

# Simple function decorator
def my_decorator(func):
    return func

@my_decorator
def simple():
    pass

# Decorator with arguments
def with_args(arg1, arg2):
    def decorator(func):
        return func
    return decorator

@with_args("hello", "world")
def decorated():
    pass

# Stacked decorators
@my_decorator
@with_args("a", "b")
def stacked():
    pass

# Class decorator
@my_decorator
class MyClass:
    pass

# Decorator with complex expression
decorators = [my_decorator]

@decorators[0]
def indexed_decorator():
    pass

# Method decorators
class Example:
    @staticmethod
    def static_method():
        pass

    @classmethod
    def class_method(cls):
        pass

    @property
    def prop(self):
        return 42
