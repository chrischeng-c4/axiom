# enumerate patterns broad

# basic
for i, v in enumerate(["a", "b", "c"]):
    print(i, v)

# with start
for i, v in enumerate(["x", "y", "z"], start=1):
    print(i, v)

# over range
for i, v in enumerate(range(5)):
    print(i, v)

# over string
for i, c in enumerate("abc"):
    print(i, c)

# list() of enumerate
print(list(enumerate(["a", "b", "c"])))
print(list(enumerate(["x", "y"], 100)))

# use in conditions
found = -1
for i, v in enumerate([3, 1, 4, 1, 5]):
    if v == 4:
        found = i
        break
print(found)

# first_index helper
def first_index(lst, target):
    for i, v in enumerate(lst):
        if v == target:
            return i
    return -1

print(first_index(["a", "b", "c", "b"], "b"))
print(first_index([1, 2, 3], 99))

# sum of indexed products
total = 0
for i, v in enumerate([2, 3, 4]):
    total += i * v
print(total)
