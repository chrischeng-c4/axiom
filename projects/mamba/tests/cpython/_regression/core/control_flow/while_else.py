x = 0
while x < 3:
    print(x)
    x += 1
else:
    print("done")

y = 0
while y < 3:
    if y == 1:
        break
    print(y)
    y += 1
else:
    print("should not print")
