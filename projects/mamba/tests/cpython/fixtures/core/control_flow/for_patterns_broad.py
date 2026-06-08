# for-loop patterns broad

# range for
total = 0
for i in range(10):
    total += i
print(total)

# range with start/stop/step
total = 0
for i in range(5, 15, 2):
    total += i
print(total)

# for over list
names = ["alice", "bob", "carol"]
for n in names:
    print(n)

# for over tuple
for x in (10, 20, 30):
    print(x)

# for over string
for c in "hey":
    print(c)

# for over dict keys
d = {"a": 1, "b": 2, "c": 3}
ks = []
for k in d:
    ks.append(k)
print(sorted(ks))

# for over dict.items()
out = []
for k, v in d.items():
    out.append((k, v))
print(sorted(out))

# for over zip
a = [1, 2, 3]
b = ["x", "y", "z"]
for x, y in zip(a, b):
    print(x, y)

# for over enumerate
for i, v in enumerate(["a", "b", "c"]):
    print(i, v)

# for with break
out = []
for i in range(10):
    if i == 5:
        break
    out.append(i)
print(out)

# for with continue
out = []
for i in range(10):
    if i % 2 == 0:
        continue
    out.append(i)
print(out)

# for-else (runs when no break)
for i in range(3):
    pass
else:
    print("else ran")

# for-else (skipped after break)
for i in range(3):
    if i == 1:
        break
else:
    print("not shown")
print("after for-else")

# nested for
total = 0
for i in range(3):
    for j in range(3):
        total += i * j
print(total)

# nested for with inner break
for i in range(3):
    for j in range(3):
        if j == 1:
            break
        print(i, j)

# accumulate via append
evens = []
for i in range(10):
    if i % 2 == 0:
        evens.append(i)
print(evens)

# for over sorted
vals = [3, 1, 4, 1, 5, 9, 2, 6]
for v in sorted(vals):
    print(v)

# for with range, compute list
sq = []
for i in range(5):
    sq.append(i * i)
print(sq)

# for over list with index via enumerate
items = ["a", "b", "c"]
out = []
for i, s in enumerate(items):
    out.append(str(i) + ":" + s)
print(out)

# early exit with flag
found = -1
for i, v in enumerate([10, 20, 30, 40]):
    if v == 30:
        found = i
        break
print(found)
