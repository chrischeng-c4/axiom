# __iter__ may return any iterator: generator, built-in iterator, or self.
# Regression: if __iter__ returned iter([...]), the backing list was freed
# when __iter__'s frame cleanup ran, leaving the iterator dangling.

class UsesGenerator:
    def __iter__(self):
        print("UsesGenerator.__iter__")
        yield 10
        yield 20

class UsesBuiltinIter:
    def __iter__(self):
        print("UsesBuiltinIter.__iter__")
        return iter([100, 200, 300])

class UsesTupleIter:
    def __iter__(self):
        print("UsesTupleIter.__iter__")
        return iter((1, 2))

for v in UsesGenerator():
    print("G:", v)

for v in UsesBuiltinIter():
    print("B:", v)

for v in UsesTupleIter():
    print("T:", v)

# Function returning a built-in iterator must also keep the list alive.
def make_iter():
    return iter([7, 8, 9])

it = make_iter()
for v in it:
    print("F:", v)

# list() on a class whose __iter__ returns a built-in iterator.
print(list(UsesBuiltinIter()))
