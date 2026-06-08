# Language conformance: functions as values (R4.6).
# Tests storing and calling functions as first-class values.

# Function stored in variable
def add(a, b):
    return a + b

f = add
print(f(3, 4))

# Functions in a dictionary
ops = {"add": add}
print(ops["add"](10, 20))

# Nested function definition
def outer():
    def inner(x):
        return x * 3
    return inner(4)

print(outer())

# Multiple nested functions
def make_pair():
    def first():
        return 1
    def second():
        return 2
    return first() + second()

print(make_pair())

# Higher-order: function returns a function
def make_multiplier(factor):
    def multiply(x):
        return x * factor
    return multiply

triple = make_multiplier(3)
print(triple(7))
