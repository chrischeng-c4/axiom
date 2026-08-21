# range usage patterns broad

# range(n)
total = 0
for i in range(10):
    total += i
print(total)

# range(start, stop)
for i in range(3, 8):
    print(i)

# range(start, stop, step)
for i in range(0, 20, 5):
    print(i)

# range to list
print(list(range(5)))
print(list(range(1, 6)))
print(list(range(0, 10, 2)))
print(list(range(0, 10, 3)))

# len of range
print(len(range(10)))

# range empty
print(list(range(0)))
print(list(range(5, 5)))
print(list(range(10, 5)))

# nested range
total = 0
for i in range(3):
    for j in range(3):
        total += i * j
print(total)

# range in sum
print(sum(range(10)))
print(sum(range(1, 11)))
print(sum(range(0, 100, 10)))

# range with if
count = 0
for i in range(20):
    if i % 3 == 0:
        count += 1
print(count)

# range in comprehension
print([x * x for x in range(5)])
print([x for x in range(10) if x % 2 == 0])

# range(0)
print(list(range(0)))

# big range
print(sum(range(100)))

# range as iterable check
r = range(5)
print(list(r))
print(list(r))  # re-iterable

# range(1) vs range(0, 1)
print(list(range(1)))
print(list(range(0, 1)))
print(list(range(0, 1, 1)))
