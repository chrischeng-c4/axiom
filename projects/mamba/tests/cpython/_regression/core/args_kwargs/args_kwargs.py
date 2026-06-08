# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# *args basic: collect extra positional args into a tuple
def variadic(*args):
    return len(args)

print(variadic())
print(variadic(1, 2, 3))

# mixed regular + *args
def mixed(a, b, *args):
    return a + b + len(args)

print(mixed(10, 20))
print(mixed(10, 20, 30, 40))

# **kwargs basic: collect extra keyword args into a dict
def kwonly(**kwargs):
    return len(kwargs)

print(kwonly())
print(kwonly(x=1, y=2))

# mixed regular + **kwargs
def mixed_kw(a, **kwargs):
    return a + len(kwargs)

print(mixed_kw(10))
print(mixed_kw(10, x=1, y=2))

# all together: regular + *args + **kwargs
def all_params(a, *args, **kwargs):
    return a + len(args) + len(kwargs)

print(all_params(100))
print(all_params(100, 1, 2, x=3, y=4))

# *args access individual elements
def sum_args(*args):
    total = 0
    for x in args:
        total = total + x
    return total

print(sum_args(1, 2, 3, 4, 5))

# **kwargs with default params
def with_defaults(a, b=10, **kwargs):
    return a + b + len(kwargs)

print(with_defaults(1))
print(with_defaults(1, b=20))
print(with_defaults(1, b=20, c=30))
