# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# kwargs and defaults broad

# positional
def f1(a, b, c):
    return a + b + c

print(f1(1, 2, 3))

# with defaults
def f2(a, b, c=10):
    return a + b + c

print(f2(1, 2))
print(f2(1, 2, 100))

# all defaults
def f3(a=1, b=2, c=3):
    return a + b + c

print(f3(10, 20, 30))
print(f3(10, 20))
print(f3(100))

# keyword args
def f4(a, b, c):
    return a * 100 + b * 10 + c

print(f4(1, 2, 3))
print(f4(a=1, b=2, c=3))
print(f4(1, 2, c=3))
print(f4(1, b=2, c=3))
print(f4(c=3, b=2, a=1))

# mix positional + kwarg
def f5(x, y=10, z=20):
    return x + y + z

print(f5(1))
print(f5(1, 2))
print(f5(1, 2, 3))
print(f5(1, z=100))
print(f5(1, y=50, z=60))

# *args
def sum_all(*args):
    total = 0
    for a in args:
        total += a
    return total

print(sum_all())
print(sum_all(1))
print(sum_all(1, 2, 3))
print(sum_all(1, 2, 3, 4, 5, 6, 7, 8, 9, 10))

# unpacking into positional
def add3(a, b, c):
    return a + b + c

args = [1, 2, 3]
print(add3(*args))

tup = (10, 20, 30)
print(add3(*tup))


