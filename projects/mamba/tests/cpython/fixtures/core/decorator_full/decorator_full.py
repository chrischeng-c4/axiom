# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Decorator conformance: function as first-class value and decorators (P0-R3).

def add(a, b):
    return a + b

print(add(3, 4))
print(add(10, 20))

# Store function in variable
f = add
print(f(5, 6))

# Pass function as argument
def call_with_args(func, a, b):
    return func(a, b)

print(call_with_args(add, 1, 2))

# Identity decorator (returns function unchanged)
def identity(func):
    return func

wrapped = identity(add)
print(wrapped(7, 8))
