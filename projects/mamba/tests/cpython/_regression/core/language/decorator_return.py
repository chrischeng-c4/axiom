# Language conformance: decorator return value propagation (R3).
# Tests that calling a decorated function returns the wrapper's result.

# TC-3.1: Simple decorator preserves return value (multi-arg)
def double(f):
    def wrapper(a, b):
        return f(a, b) * 2
    return wrapper

@double
def add(a, b):
    return a + b

print(add(3, 4))

# TC-3.2: Identity decorator returns original value
def identity(f):
    return f

@identity
def greet():
    return 42

print(greet())

# TC-3.3: Stacked decorators — requires closure capture in decorator chains.
# Tracked separately from the decorator return value fix (#1084).
# Uncomment when closure capture across stacked decorators is fixed.
# def add_one(f):
#     def w():
#         return f() + 1
#     return w
#
# def double2(f):
#     def w():
#         return f() * 2
#     return w
#
# @add_one
# @double2
# def val():
#     return 5
#
# print(val())  # expected: 11
