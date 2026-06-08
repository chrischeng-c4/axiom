# loop patterns broad

# for basic
total = 0
for i in [1, 2, 3, 4, 5]:
    total += i
print(total)

# for range
total = 0
for i in range(100):
    total += i
print(total)

# for with break
for i in range(100):
    if i == 5:
        break
    print(i)

# for with continue
for i in range(10):
    if i % 2 == 0:
        continue
    print(i)

# while basic
n = 10
while n > 0:
    print(n)
    n -= 1

# while with break
n = 0
while True:
    if n >= 3:
        break
    print(n)
    n += 1

# while continue
n = 0
total = 0
while n < 10:
    n += 1
    if n % 2 == 0:
        continue
    total += n
print(total)

# for with else (no break)
for i in range(3):
    print("in", i)
else:
    print("done")

# for with else (with break)
for i in range(5):
    if i == 3:
        break
    print("in2", i)
else:
    print("done2")

# nested for
for i in range(3):
    for j in range(3):
        print(i, j)

# nested with break inner
for i in range(3):
    for j in range(3):
        if j == 2:
            break
        print("inner", i, j)

# counter with for
counts = 0
for c in "hello world":
    if c == "l":
        counts += 1
print(counts)

# for over dict
d = {"a": 1, "b": 2, "c": 3}
total = 0
for k in sorted(d):
    total += d[k]
print(total)

# for over items
for k, v in sorted(d.items()):
    print(k, v)

# reversed iter
for x in reversed([1, 2, 3, 4, 5]):
    print(x)

for c in reversed("abc"):
    print(c)

# enumerate
for i, v in enumerate(["x", "y", "z"]):
    print(i, v)

# enumerate with start
for i, v in enumerate(["a", "b"], start=10):
    print(i, v)
