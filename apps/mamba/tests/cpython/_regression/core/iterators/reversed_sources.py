# reversed() on various sequence types

# list
print(list(reversed([1, 2, 3])))

# tuple
print(list(reversed((1, 2, 3))))

# range
print(list(reversed(range(5))))

# string
print(list(reversed("abc")))

# bytes yields int codes
print(list(reversed(b"AB")))

# empty sources
print(list(reversed([])))
print(list(reversed("")))

# single element
print(list(reversed([42])))
print(list(reversed("x")))

# used in a for-loop
result = []
for x in reversed([1, 2, 3, 4]):
    result.append(x)
print(result)
