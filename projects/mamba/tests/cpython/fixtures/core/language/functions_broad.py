# function features broad

# positional / keyword args
def greet(name, greeting="Hello"):
    return f"{greeting}, {name}!"

print(greet("Alice"))
print(greet("Bob", "Hi"))
print(greet(name="Carol"))
print(greet(greeting="Hey", name="Dan"))

# *args
def sum_all(*args):
    return sum(args)

print(sum_all(1, 2, 3))
print(sum_all(1, 2, 3, 4, 5))
print(sum_all())

# **kwargs
def describe(**kwargs):
    pairs = [f"{k}={v}" for k, v in sorted(kwargs.items())]
    return ", ".join(pairs)

print(describe(a=1, b=2))
print(describe(x=10))
print(describe())

# combined positional + *args
def head_and_rest(first, *rest):
    return first, list(rest)

print(head_and_rest(1, 2, 3, 4))
print(head_and_rest(1))

# unpacking into function call
nums = [1, 2, 3]
def add3(a, b, c):
    return a + b + c
print(add3(*nums))

# returning multiple values (tuple)
def stats(xs):
    return min(xs), max(xs), sum(xs) / len(xs)

lo, hi, avg = stats([2, 4, 6, 8])
print(lo, hi, avg)

# recursive
def fact(n):
    if n <= 1:
        return 1
    return n * fact(n - 1)

print(fact(5))
print(fact(10))

# function with docstring (no crash)
def doc_fn():
    """This is a docstring"""
    return 42

print(doc_fn())

# lambda as arg
print(sorted([3, -1, 4, -1, 5, -9, 2, -6], key=lambda x: abs(x)))

# filter+map
print(list(filter(lambda x: x > 0, [-2, -1, 0, 1, 2])))
print(list(map(lambda x: x * 10, [1, 2, 3])))
