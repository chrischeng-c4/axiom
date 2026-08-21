# Custom iterator class — exercises the iterator protocol surface that
# user-defined classes hit through `__iter__` / `__next__`: for-loop
# iteration, list() materialization, next() (with and without default),
# `in` membership, and tuple unpacking.

class CountDown:
    def __init__(self, start):
        self.current = start

    def __iter__(self):
        return self

    def __next__(self):
        if self.current <= 0:
            raise StopIteration
        val = self.current
        self.current = self.current - 1
        return val

# for loop over custom iterator
for x in CountDown(5):
    print(x)

# Multiple iterations create separate instances
for x in CountDown(3):
    print(x)

# list() materialization
print(list(CountDown(3)))

# next() — with and without default once exhausted
it = CountDown(2)
print(next(it))
print(next(it))
print(next(it, "done"))

# `in` membership probe
print(2 in CountDown(5))
print(99 in CountDown(5))

# tuple unpacking
a, b, c = CountDown(3)
print(a, b, c)
