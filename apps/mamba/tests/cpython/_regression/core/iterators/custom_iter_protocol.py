# Custom __iter__ / __next__ protocol + iter()/next() builtins

class Counter:
    def __init__(self, limit):
        self.limit = limit
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= self.limit:
            raise StopIteration
        self.i = self.i + 1
        return self.i

# for-loop over custom iterator
c = Counter(3)
for x in c:
    print(x)

# Fresh instance for second iteration
for x in Counter(2):
    print("b", x)

# next() directly on custom iterator
c2 = Counter(3)
print(next(c2))
print(next(c2))

# iter()/next() on a list
it = iter([10, 20])
print(next(it))
print(next(it))
try:
    v = next(it)
except StopIteration:
    print("stopped")

# list() consumes an iterator
print(list(Counter(4)))
