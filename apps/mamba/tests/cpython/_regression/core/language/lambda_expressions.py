# Language conformance: lambda expressions (P0-R1).
# Tests lambda compilation, closure capture, and nested lambdas.

# Simple lambda
square = lambda x: x * x
print(square(5))

# Lambda with multiple args
add = lambda x, y: x + y
print(add(3, 4))

# Nested lambda (closure)
adder = lambda x: lambda y: x + y
add3 = adder(3)
print(add3(4))

# Lambda used with map
nums = list(map(lambda x: x * 2, [1, 2, 3]))
print(nums)

# Lambda used with filter
evens = list(filter(lambda x: x % 2 == 0, range(10)))
print(evens)

# Lambda in list
ops = [lambda x: x + 1, lambda x: x * 2]
print(ops[0](5))
print(ops[1](5))
