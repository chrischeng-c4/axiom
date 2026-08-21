# lambda deep broad

# basic
sq = lambda x: x * x
print(sq(5))
print(sq(10))

# multi-arg
add = lambda a, b: a + b
print(add(2, 3))
print(add(10, 20))

# no args
get1 = lambda: 1
print(get1())

# in map
print(list(map(lambda x: x * 2, [1, 2, 3, 4])))
print(list(map(lambda x: x + 10, range(5))))

# in filter
print(list(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))
print(list(filter(lambda x: x % 2 == 0, range(10))))

# in sort key
pairs = [(1, "b"), (2, "a"), (3, "c")]
print(sorted(pairs, key=lambda p: p[1]))

# immediate invocation
print((lambda x: x + 1)(10))

# conditional inside lambda
sign = lambda x: "pos" if x > 0 else ("neg" if x < 0 else "zero")
print(sign(5))
print(sign(-5))
print(sign(0))

# lambda as arg
def apply(f, x):
    return f(x)

print(apply(lambda n: n * 3, 7))
print(apply(lambda n: n + 100, 1))
