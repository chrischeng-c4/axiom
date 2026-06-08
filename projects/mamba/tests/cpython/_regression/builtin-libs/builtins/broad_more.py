# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# built-in functions broad coverage

# repr / str
print(repr("hello"))
print(repr(42))
print(repr([1, 2, 3]))
print(repr((1, 2)))
print(str("hello"))
print(str(42))
print(str([1, 2]))

# id / hash (not ==, just type checks)
print(type(id(42)).__name__)
print(type(hash("x")).__name__)

# len on various
print(len("hello"))
print(len([1, 2, 3]))
print(len((1, 2, 3)))
print(len({"a": 1, "b": 2}))
print(len({1, 2, 3}))

# sorted with mixed
print(sorted([3, 1, 2]))
print(sorted(["banana", "apple", "cherry"]))
print(sorted([(1, "b"), (2, "a"), (1, "a")]))
print(sorted([3, 1, 2], reverse=True))
print(sorted(["abc", "de", "f"], key=len))
print(sorted({"b": 2, "a": 1, "c": 3}.items()))

# reversed
print(list(reversed([1, 2, 3])))
print(list(reversed("abc")))
print(list(reversed((1, 2, 3))))
print(list(reversed(range(5))))

# map / filter / list
print(list(map(lambda x: x * 2, [1, 2, 3])))
print(list(map(str, [1, 2, 3])))
print(list(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))
print(list(filter(None, [0, 1, 0, 2, 3, 0])))

# zip
print(list(zip([1, 2, 3], ["a", "b", "c"])))
print(list(zip([1, 2, 3], [10, 20, 30], [100, 200, 300])))
print(list(zip("abc", "xyz")))

# enumerate
print(list(enumerate("abc")))
print(list(enumerate("abc", start=10)))

# sum with start
print(sum([1, 2, 3]))
print(sum([1, 2, 3], 100))

# all / any
print(all([1, 2, 3]))
print(all([0, 1, 2]))
print(any([0, 0, 1]))
print(any([0, 0, 0]))

# print options
print("a", "b", "c")
print("a", "b", "c", sep="-")
print("a", "b", "c", end="!\n")

# type / type() call
print(type(42).__name__)
print(type("x").__name__)
print(type([]).__name__)
print(type((1,)).__name__)
print(type({}).__name__)
print(type({1}).__name__)
print(type(None).__name__)
print(type(True).__name__)

# callable
def f(): pass
print(callable(f))
print(callable(42))
print(callable(print))
