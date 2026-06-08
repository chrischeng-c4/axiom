# RUN: parse

@my_decorator
def foo() -> int:
    return 1

@decorator1
@decorator2
def bar() -> int:
    return 2
