# Iterator protocol: __iter__, __next__
class Counter:
    def __init__(self, limit):
        self.limit = limit
        self.current = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.current >= self.limit:
            raise StopIteration
        val = self.current
        self.current += 1
        return val

# Use in for loop
for x in Counter(5):
    print(x)

# Manual iteration
it = iter(Counter(3))
print(next(it))
print(next(it))
print(next(it))
try:
    next(it)
except StopIteration:
    print("StopIteration raised")

# iter() on built-in types
print(list(iter([10, 20, 30])))
print(list(iter((1, 2, 3))))
print(list(iter("abc")))

# next() with default
it2 = iter([1])
print(next(it2, "default"))
print(next(it2, "default"))
