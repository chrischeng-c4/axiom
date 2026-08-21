# lambda patterns broad

# basic lambda
add = lambda a, b: a + b
print(add(1, 2))
print(add(10, 20))

sq = lambda x: x * x
print(sq(5))
print(sq(10))

# lambda as argument
def apply(fn, x):
    return fn(x)

print(apply(lambda x: x + 1, 10))
print(apply(lambda x: x * x, 5))

# lambda with multiple args
mul = lambda a, b, c: a * b * c
print(mul(2, 3, 4))

# lambda with multi positional
inc = lambda x, step: x + step
print(inc(10, 1))
print(inc(10, 5))

# lambda in sort key
data = [(1, "b"), (2, "a"), (3, "c")]
sorted_by_str = sorted(data, key=lambda t: t[1])
print(sorted_by_str)

# lambda returning tuple
swap = lambda a, b: (b, a)
print(swap(1, 2))
print(swap("x", "y"))

# lambda in filter
nums = [1, 2, 3, 4, 5, 6]
evens = list(filter(lambda x: x % 2 == 0, nums))
print(evens)

# lambda in map
doubled = list(map(lambda x: x * 2, nums))
print(doubled)

# lambda with condition (ternary)
sign = lambda n: "+" if n > 0 else "-" if n < 0 else "0"
print(sign(5))
print(sign(-3))
print(sign(0))

# lambda in sorted key for dict values
d = {"a": 3, "b": 1, "c": 2}
sorted_by_v = sorted(d.items(), key=lambda kv: kv[1])
print(sorted_by_v)

# lambda with no args
forty_two = lambda: 42
print(forty_two())
