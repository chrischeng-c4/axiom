# augmented assignment broad

# int ops
x = 10
x += 5
print(x)
x -= 3
print(x)
x *= 2
print(x)
x //= 4
print(x)
x %= 5
print(x)
x **= 3
print(x)

# from 0
y = 0
y += 100
print(y)
y -= 50
print(y)
y *= 2
print(y)

# chained augmented
a = 1
a += 1
a += 1
a += 1
print(a)

# list augmented
li = [1, 2, 3]
li += [4, 5]
print(li)

# str augmented
s = "hello"
s += " world"
print(s)
s += "!"
print(s)

# * (repetition)
s2 = "ab"
s2 *= 3
print(s2)

# += in loop
total = 0
for i in range(10):
    total += i
print(total)

# augmented with negative
n = 10
n += -5
print(n)
n -= -3
print(n)

# divmod style
d = 100
d //= 3
print(d)
d %= 2
print(d)

# in function
def acc(n):
    total = 0
    for i in range(n):
        total += i
    return total

print(acc(5))
print(acc(10))

