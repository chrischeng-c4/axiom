# list
print(list(reversed([1, 2, 3, 4, 5])))
print(list(reversed([])))
print(list(reversed([42])))

# range
print(list(reversed(range(5))))
print(list(reversed(range(2, 10))))
print(list(reversed(range(0, 10, 2))))
print(list(reversed(range(10, 0, -2))))

# string
print(list(reversed("hello")))
print("".join(reversed("world")))

# tuple
print(list(reversed((10, 20, 30))))

# enumerate with reversed
for i, x in enumerate(reversed([100, 200, 300])):
    print(i, x)

# reversed produces iterator
it = reversed([1, 2, 3])
print(next(it))
print(next(it))
print(next(it))

# sum over reversed
print(sum(reversed([1, 2, 3, 4, 5])))

# reversed of sorted
print(list(reversed(sorted([3, 1, 4, 1, 5, 9, 2, 6]))))
