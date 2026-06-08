# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# function args deeper broad

# int defaults
def power(base, exp=2):
    return base ** exp

print(power(3))
print(power(3, 3))
print(power(2, 10))
print(power(base=5))
print(power(base=5, exp=3))
print(power(5, exp=3))

# multi-int defaults
def info(a, b, c=10, d=20):
    return [a, b, c, d]

print(info(1, 2))
print(info(1, 2, 3))
print(info(1, 2, 3, 4))
print(info(1, 2, d=99))
print(info(1, 2, c=7, d=8))
print(info(a=1, b=2, c=3, d=4))

# **kwargs
def pass_kw(**kw):
    return sorted(kw.items())

print(pass_kw(a=1, b=2, c=3))
print(pass_kw())

# *args
def pass_args(*a):
    return list(a)

print(pass_args(1, 2, 3))
print(pass_args())
print(pass_args(42))

# lots of positional kwargs
def many(a, b, c, d, e):
    return [a, b, c, d, e]

print(many(1, 2, 3, 4, 5))
print(many(e=5, d=4, c=3, b=2, a=1))
print(many(1, 2, e=5, c=3, d=4))

# unpacking positional list into call
def three(a, b, c):
    return [a, b, c]

args = [1, 2, 3]
print(three(*args))

t = (10, 20, 30)
print(three(*t))
