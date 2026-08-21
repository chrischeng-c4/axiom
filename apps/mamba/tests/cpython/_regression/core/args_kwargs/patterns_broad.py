# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# args/kwargs patterns broad

# *args basic
def sumall(*args):
    total = 0
    for a in args:
        total += a
    return total

print(sumall())
print(sumall(1))
print(sumall(1, 2, 3))

# **kwargs basic
def kw(**kwargs):
    return sorted(kwargs.items())

print(kw())
print(kw(a=1))
print(kw(a=1, b=2, c=3))

# mixed positional + *args
def mixed(a, b, *rest):
    return (a, b, list(rest))

print(mixed(1, 2))
print(mixed(1, 2, 3))

# defaults
def defaulted(a, b=10, c=20):
    return a + b + c

print(defaulted(1))
print(defaulted(1, 2))
print(defaulted(1, 2, 3))

# keyword args in call
def named(a, b, c):
    return (a, b, c)

print(named(1, 2, 3))
print(named(a=1, b=2, c=3))
print(named(1, c=3, b=2))
