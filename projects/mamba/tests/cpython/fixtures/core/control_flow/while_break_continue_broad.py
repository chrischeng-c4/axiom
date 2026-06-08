# while/break/continue/else broad

# basic while
i = 0
while i < 5:
    print(i)
    i += 1

# while with break
i = 0
while True:
    if i >= 5:
        break
    print(i)
    i += 1
print("after")

# while with continue
i = 0
while i < 6:
    i += 1
    if i % 2 == 0:
        continue
    print(i)

# nested break
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

# while with else (no-break)
i = 0
while i < 3:
    print("loop", i)
    i += 1
else:
    print("else-no-break")

# while with else (break suppresses)
i = 0
while i < 5:
    if i == 2:
        break
    print("loop2", i)
    i += 1
else:
    print("else-should-not-run")
print("after")

# for with else (no-break)
for x in [1, 2, 3]:
    print("for", x)
else:
    print("for-else-ran")

# for with else (break suppresses)
for x in [1, 2, 3]:
    if x == 2:
        break
    print("for2", x)
else:
    print("for-else-shouldnot")
print("after")

# infinite while with break
count = 0
while True:
    count += 1
    if count >= 3:
        break
print("count=", count)

# while with compound cond
a = 0
b = 10
while a < 5 and b > 5:
    a += 1
    b -= 1
print(a, b)

# while countdown
n = 5
while n > 0:
    print("down", n)
    n -= 1
