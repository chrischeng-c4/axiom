# Ternary conditional expression

# Basic ternary
x = "yes" if True else "no"
print(x)

y = "yes" if False else "no"
print(y)

# Ternary with computation
a = 10
b = 20
bigger = a if a > b else b
print(bigger)

# Nested ternary
val = 15
label = "small" if val < 10 else "medium" if val < 20 else "large"
print(label)

# Ternary with None
result = None if False else 42
print(result)

# Mixed type: str/int
mixed = "str" if False else 42
print(mixed)
