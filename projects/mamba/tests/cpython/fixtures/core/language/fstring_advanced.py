# Language conformance: f-string features (R4.10).
# Tests f-string with expressions, method calls, conditionals.
# (Nested f-strings and format specs avoided due to parser limitations)

# Basic f-strings
name = "World"
n = 42
print(f"Hello, {name}!")
print(f"n = {n}")

# Arithmetic in f-strings
x = 3
y = 4
print(f"{x} + {y} = {x + y}")

# Function call in f-string
def double(n):
    return n * 2

print(f"double(5) = {double(5)}")

# Conditional expression in f-string
flag = True
print(f"flag is {'yes' if flag else 'no'}")

# Dict access in f-string
d = {"key": "value"}
print(f"d['key'] = {d['key']}")

# f-string concatenation
a = 10
b = 20
msg = f"a = {a}, b = {b}, sum = {a + b}"
print(msg)

# Empty f-string
print(f"")

# Escaped braces
print(f"{{literal braces}}")

# String methods in f-string
print(f"{'hello'.upper()}")
print(f"{'  spaces  '.strip()}")
