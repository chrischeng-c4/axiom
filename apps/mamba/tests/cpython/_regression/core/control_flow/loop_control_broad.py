# control flow loop control broad

# break basic
for i in range(10):
    if i == 5:
        break
    print(i)

# continue basic
for i in range(6):
    if i % 2 == 0:
        continue
    print(i)

# break in while
x = 0
while True:
    if x >= 3:
        break
    print(x)
    x += 1

# continue in while
x = 0
while x < 6:
    x += 1
    if x % 2 == 0:
        continue
    print(x)

# nested break (only inner)
for i in range(3):
    for j in range(3):
        if j == 2:
            break
        print(i, j)

# nested continue
for i in range(3):
    for j in range(3):
        if j == 1:
            continue
        print(i, j)

# early return
def first_gt(n, lst):
    for v in lst:
        if v > n:
            return v
    return None

print(first_gt(3, [1, 2, 3, 4, 5]))
print(first_gt(10, [1, 2, 3]))

# for-else (runs when no break)
for i in range(3):
    print(i)
else:
    print("done")

# for-else (break skips else)
for i in range(3):
    if i == 1:
        break
    print(i)
else:
    print("NOT REACHED")

print("after")
