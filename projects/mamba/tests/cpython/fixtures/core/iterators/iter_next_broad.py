# iter/next builtin broad

# iter over list
it = iter([1, 2, 3])
print(next(it))
print(next(it))
print(next(it))

# iter over string
it2 = iter("abc")
print(next(it2))
print(next(it2))
print(next(it2))

# iter over tuple
it3 = iter((10, 20, 30))
print(next(it3))
print(next(it3))

# iter w/ default
it4 = iter([1, 2])
print(next(it4, -1))
print(next(it4, -1))
print(next(it4, -1))
print(next(it4, -1))

# iter w/ default string
it5 = iter(["a"])
print(next(it5, "DONE"))
print(next(it5, "DONE"))

# for loop on iter
for x in iter([100, 200, 300]):
    print(x)

# iter over range
it6 = iter(range(3))
print(next(it6))
print(next(it6))
print(next(it6))

# partial iter consumption
data = [1, 2, 3, 4, 5]
it7 = iter(data)
print(next(it7))
print(next(it7))
remaining = list(it7)
print(remaining)

# iter + for loop partial
it8 = iter([1, 2, 3, 4, 5])
print(next(it8))
for x in it8:
    print(x)
    if x == 3:
        break

# list from iter
print(list(iter("hello")))
print(list(iter([1, 2, 3])))
print(list(iter(range(5))))

# iter on generator
def gen():
    yield 1
    yield 2
    yield 3

g = gen()
print(next(g))
print(next(g))
print(next(g))
