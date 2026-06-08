# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# generator patterns broad

# basic generator
def gen1():
    yield 1
    yield 2
    yield 3

for x in gen1():
    print(x)

# list from generator
print(list(gen1()))

# sum over generator
def nums(n):
    i = 0
    while i < n:
        yield i
        i += 1

print(sum(nums(5)))
print(sum(nums(10)))

# max/min over generator
print(max(nums(10)))
print(min(nums(10)))

# generator with condition
def evens(n):
    i = 0
    while i < n:
        if i % 2 == 0:
            yield i
        i += 1

print(list(evens(10)))
print(sum(evens(20)))

# generator with argument
def multiples(base, count):
    i = 1
    while i <= count:
        yield base * i
        i += 1

print(list(multiples(3, 5)))
print(list(multiples(7, 3)))

# generator from list
def from_list(lst):
    for x in lst:
        yield x * 2

print(list(from_list([1, 2, 3, 4])))
print(list(from_list([])))

# generator filter
def only_pos(lst):
    for x in lst:
        if x > 0:
            yield x

print(list(only_pos([-2, -1, 0, 1, 2])))

# generator w/ ternary yield
def abs_gen(lst):
    for x in lst:
        yield x if x >= 0 else -x

print(list(abs_gen([-3, -1, 0, 2, 5])))

# chained generators
def times2(gen):
    for x in gen:
        yield x * 2

print(list(times2(nums(5))))
print(list(times2(evens(10))))

# generator expression vs function
g = (x * x for x in range(5))
print(list(g))
g2 = (x + 1 for x in [10, 20, 30])
print(list(g2))

# any/all on generator
print(any(x > 3 for x in nums(5)))
print(all(x >= 0 for x in nums(5)))
print(any(x < 0 for x in nums(5)))
