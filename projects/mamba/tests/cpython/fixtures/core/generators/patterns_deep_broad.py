# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# generator usage patterns deep broad

# simple count
def count_up(n):
    i = 0
    while i < n:
        yield i
        i += 1

for x in count_up(5):
    print(x)

# list from gen
def gen_squares(n):
    for i in range(n):
        yield i * i

print(list(gen_squares(5)))

# filter via generator
def evens(items):
    for x in items:
        if x % 2 == 0:
            yield x

print(list(evens([1, 2, 3, 4, 5, 6, 7, 8])))

# transform via generator
def doubled(items):
    for x in items:
        yield x * 2

print(list(doubled([1, 2, 3, 4, 5])))

# genexp
print(list(x * x for x in range(6)))
print(sum(x for x in range(10)))
print(sum(x for x in range(10) if x % 2 == 0))

# fibonacci via temp swap
def fib2(n):
    a = 0
    b = 1
    count = 0
    while count < n:
        yield a
        t = a + b
        a = b
        b = t
        count += 1

print(list(fib2(10)))

# early termination
def take(gen, n):
    count = 0
    for x in gen:
        if count >= n:
            return
        yield x
        count += 1

print(list(take(count_up(100), 5)))

# generator chaining
def first_half(items):
    half = len(items) // 2
    for i in range(half):
        yield items[i]

data = [1, 2, 3, 4, 5, 6]
print(list(first_half(data)))
