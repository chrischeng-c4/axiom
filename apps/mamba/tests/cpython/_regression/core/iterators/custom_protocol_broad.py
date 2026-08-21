# custom iterator protocol broad

# basic __iter__ / __next__ (self-iter)
class Countdown:
    def __init__(self, n):
        self.n = n
    def __iter__(self):
        return self
    def __next__(self):
        if self.n <= 0:
            raise StopIteration
        self.n -= 1
        return self.n + 1

c = Countdown(5)
for x in c:
    print(x)

# another self-iter
class UpTo:
    def __init__(self, limit):
        self.i = 0
        self.limit = limit
    def __iter__(self):
        return self
    def __next__(self):
        if self.i >= self.limit:
            raise StopIteration
        v = self.i
        self.i += 1
        return v

for x in UpTo(3):
    print(x)

# iter() on list
li = iter([10, 20, 30])
print(next(li))
print(next(li))
print(next(li))

# next with default
ix = iter([])
print(next(ix, -1))
print(next(ix, "done"))

# sum on self-iter
print(sum(Countdown(5)))  # 5+4+3+2+1 = 15

# list on self-iter
print(list(Countdown(4)))
