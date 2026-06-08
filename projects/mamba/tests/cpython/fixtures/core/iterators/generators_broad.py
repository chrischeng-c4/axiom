# iterator / generator broad

# simple generator
def counter(n):
    i = 0
    while i < n:
        yield i
        i += 1

print(list(counter(5)))
print(sum(counter(10)))

# generator with conditions
def evens(n):
    for i in range(n):
        if i % 2 == 0:
            yield i

print(list(evens(10)))

# yield from
def chain_ab():
    yield from [1, 2, 3]
    yield from (4, 5)

print(list(chain_ab()))

# generator with state
def running_sum(seq):
    total = 0
    for x in seq:
        total += x
        yield total

print(list(running_sum([1, 2, 3, 4, 5])))

# for-else
def find(seq, target):
    for x in seq:
        if x == target:
            return "found"
    else:
        return "not found"

print(find([1, 2, 3], 2))
print(find([1, 2, 3], 99))

# zip with different lengths
print(list(zip([1, 2, 3], ["a", "b"])))
print(list(zip([1, 2], ["a", "b"], [True, False])))

# enumerate with start
print(list(enumerate("abc")))
print(list(enumerate("abc", 10)))

# reversed
print(list(reversed([1, 2, 3])))
print(list(reversed("abc")))
print(list(reversed(range(5))))
