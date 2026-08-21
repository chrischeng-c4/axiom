# RUN: parse
# CPython 3.12 test_decorators: stacked decorator syntax

def decorator_a(func):
    return func

def decorator_b(func):
    return func

def decorator_c(arg):
    def inner(func):
        return func
    return inner

# Two stacked decorators
@decorator_a
@decorator_b
def two_stacked():
    pass

# Three stacked decorators
@decorator_a
@decorator_b
@decorator_c(42)
def three_stacked():
    pass

# Stacked on class
@decorator_a
@decorator_b
class StyledClass:
    pass

# Stacked on method
class Example:
    @decorator_a
    @decorator_b
    def method(self):
        pass

    @staticmethod
    @decorator_a
    def static_method():
        pass

    @classmethod
    @decorator_b
    def class_method(cls):
        pass
