# break with else / return
def find(items, target):
    for x in items:
        if x == target:
            return "found"
    return "missing"

print(find([1, 2, 3], 2))
print(find([1, 2, 3], 99))

# break in middle
for x in [1, 2, 3, 4, 5]:
    if x == 3:
        break
    print(x)

# continue
for x in range(10):
    if x % 2 == 0:
        continue
    print(x)

# for-else runs when loop completes without break
def all_positive(items):
    for x in items:
        if x <= 0:
            break
    else:
        return "all positive"
    return "has zero/negative"

print(all_positive([1, 2, 3]))
print(all_positive([1, -1, 3]))

# while with break
n = 10
while True:
    n -= 1
    if n == 0:
        break

print(n)

# while-else
n = 0
while n < 3:
    n += 1
else:
    print("while-else ran")

print(n)

# loop accumulate
s = 0
for x in range(1, 11):
    s += x
print(s)

# enumerate in for
for i, c in enumerate("hello"):
    print(i, c)

# zip in for
for a, b in zip([1, 2, 3], ["x", "y", "z"]):
    print(a, b)

# dict iteration
d = {"a": 1, "b": 2, "c": 3}
for k, v in sorted(d.items()):
    print(k, v)

# reversed in for
for x in reversed([1, 2, 3, 4]):
    print(x)

# sorted in for
for x in sorted([3, 1, 4, 1, 5, 9, 2, 6]):
    print(x)
