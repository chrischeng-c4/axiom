# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# yield from delegation
def inner():
    yield 1
    yield 2
    yield 3

def outer():
    yield 0
    yield from inner()
    yield 4

print(list(outer()))

# yield from with return value
def inner_with_return():
    yield 10
    yield 20
    return "done"

def outer_with_return():
    result = yield from inner_with_return()
    print("inner returned:", result)
    yield 99

print(list(outer_with_return()))

# yield from iterable (not generator)
def from_list():
    yield from [1, 2, 3]

print(list(from_list()))

# yield from string
def from_string():
    yield from "abc"

print(list(from_string()))

# Nested yield from
def a():
    yield 1

def b():
    yield from a()
    yield 2

def c():
    yield from b()
    yield 3

print(list(c()))
