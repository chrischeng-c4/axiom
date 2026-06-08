# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# iter/next on list
it = iter([1, 2, 3])
print(next(it))
print(next(it))
print(next(it))

# next with default
it = iter([1])
print(next(it))
print(next(it, "done"))

# iter on string
it = iter("hi")
print(next(it))
print(next(it))

# iter on range
it = iter(range(3))
print(next(it))
print(next(it))
print(next(it))

# iter on tuple
it = iter((10, 20, 30))
print(next(it))
print(next(it))
print(next(it))

# list from iter
print(list(iter([1, 2, 3, 4, 5])))
print(list(iter("abc")))

# reversed iterator
r = reversed([1, 2, 3])
print(next(r))
print(next(r))
print(next(r))

# iter then exhausted
it = iter([1])
print(next(it))
try:
    next(it)
except StopIteration:
    print("stop")

# iter over set
print(sorted(iter({1, 2, 3, 4, 5})))

# zip iterator (consumed in list)
print(list(zip([1, 2, 3], ["a", "b", "c"])))

# map fully consumed
print(list(map(lambda x: x * 2, [1, 2, 3])))

# filter fully consumed
print(list(filter(lambda x: x > 2, [1, 2, 3, 4])))
