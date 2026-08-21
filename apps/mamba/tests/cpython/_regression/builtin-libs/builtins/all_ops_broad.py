# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# builtins broad (beyond existing tests)

# sum
print(sum([1, 2, 3]))
print(sum([]))
print(sum((1, 2, 3)))
print(sum(range(10)))
print(sum([1, 2, 3], 100))

# min / max
print(min(1, 2, 3))
print(max(1, 2, 3))
print(min([5, 3, 8, 1]))
print(max([5, 3, 8, 1]))
print(min("banana"))
print(max("banana"))
print(min([5, 3, 8, 1], default=0))
print(min([], default=99))

# abs
print(abs(-5))
print(abs(5))
print(abs(0))
print(abs(-3.14))

# pow
print(pow(2, 10))
print(pow(2, 10, 1000))
print(pow(3, 2))

# round
print(round(3.5))
print(round(4.5))
print(round(3.14159, 2))
print(round(-3.14159, 2))

# divmod
print(divmod(17, 5))
print(divmod(10, 3))

# len
print(len([1, 2, 3]))
print(len("hello"))
print(len((1, 2)))
print(len({}))
print(len({"a": 1, "b": 2}))

# sorted
print(sorted([3, 1, 4, 1, 5, 9, 2, 6]))
print(sorted([3, 1, 4, 1, 5, 9, 2, 6], reverse=True))
print(sorted("python"))
print(sorted([(1, "b"), (2, "a"), (1, "a")]))

# enumerate
print(list(enumerate(["a", "b", "c"])))
print(list(enumerate(["a", "b", "c"], start=10)))

# zip
print(list(zip([1, 2, 3], ["a", "b", "c"])))
print(list(zip([1, 2, 3], ["a", "b", "c"], [True, False, True])))
print(list(zip([1, 2, 3], [])))

# reversed
print(list(reversed([1, 2, 3])))
print(list(reversed("abc")))

# all / any
print(all([True, True, True]))
print(all([True, False]))
print(all([]))
print(any([False, False, True]))
print(any([False, False, False]))
print(any([]))

# filter / map
print(list(filter(lambda x: x > 0, [-1, 0, 1, 2])))
print(list(map(lambda x: x + 1, [1, 2, 3])))
