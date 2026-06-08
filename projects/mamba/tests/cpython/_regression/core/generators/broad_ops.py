# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
def count_up(n):
    i = 0
    while i < n:
        yield i
        i += 1

g = count_up(5)
print(list(g))

def evens(upto):
    for i in range(upto):
        if i % 2 == 0:
            yield i

print(list(evens(10)))

def pair_items(items):
    for x in items:
        yield x
        yield x * 2

print(list(pair_items([1, 2, 3])))

# generator expression
g = (x * x for x in range(6))
print(list(g))

# gen expr in sum
print(sum(x * x for x in range(10)))

# filter in list
print([x for x in range(20) if x % 3 == 0])

# next()
g = count_up(3)
print(next(g))
print(next(g))
print(next(g))

# accumulator via temp
def sums(items):
    total = 0
    for x in items:
        total += x
        yield total

print(list(sums([1, 2, 3, 4, 5])))

# double generator
def doubled(items):
    for x in items:
        yield x * 2

print(list(doubled([5, 10, 15])))

# conditional yield
def filtered(items, thresh):
    for x in items:
        if x > thresh:
            yield x

print(list(filtered([1, 5, 2, 7, 3, 9], 3)))
