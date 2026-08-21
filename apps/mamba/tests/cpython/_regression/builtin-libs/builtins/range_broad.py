# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# basic
print(list(range(5)))
print(list(range(1, 6)))
print(list(range(0, 10, 2)))
print(list(range(10, 0, -1)))
print(list(range(10, 0, -2)))

# empty
print(list(range(0)))
print(list(range(5, 5)))
print(list(range(5, 10, -1)))

# len simple
print(len(range(10)))

# indexing
r = range(10)
print(r[0])
print(r[5])
print(r[-1])
print(r[-2])

# membership
print(5 in range(10))
print(15 in range(10))

# sum
print(sum(range(11)))
print(sum(range(1, 101)))

# zip with range
print(list(zip(range(3), ["a", "b", "c"])))

# enumerate with range
print(list(enumerate(["x", "y", "z"])))

# for-loop
total = 0
for i in range(1, 11):
    total += i
print(total)

# reversed range
print(list(reversed(range(5))))
