# S4: Nested star unpacking — outer and inner destructuring
a, (*b, c) = [1, [2, 3, 4]]
print(a)
print(b)
print(c)

# Simple nested without star
x, (y, z) = [10, [20, 30]]
print(x)
print(y)
print(z)
