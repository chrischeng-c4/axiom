# chained comparisons broad

# basic
x = 5
print(1 < x < 10)
print(1 < x < 3)
print(10 > x > 1)
print(10 > x > 5)

# equality chain
a = 1
b = 1
c = 1
print(a == b == c)

a = 1
b = 2
c = 1
print(a == b == c)

# lt/le chain
x = 5
print(0 < x <= 5)
print(0 < x < 5)
print(0 <= x <= 5)

# with variables on both sides
lo = 0
hi = 100
mid = 50
print(lo < mid < hi)
print(lo <= mid <= hi)
print(hi > mid > lo)

# float chain
f = 3.5
print(1.0 < f < 10.0)
print(5.0 < f < 10.0)

# string chain
s = "c"
print("a" < s < "z")
print("a" <= s <= "c")

# in-order chain
print(1 < 2 < 3 < 4)
print(1 < 2 < 2)
print(1 <= 2 <= 2)

# usage in if
n = 50
if 0 <= n <= 100:
    print("in range")
else:
    print("out")

# negation
n = -1
if not (0 <= n <= 100):
    print("out of range")

# ternary with chain
x = 5
print("mid" if 0 < x < 10 else "out")

# chain in while
n = 0
total = 0
while 0 <= n < 5:
    total += n
    n += 1
print(total)

# in list comp
results = [x for x in range(10) if 2 < x < 8]
print(results)

# chained != / mixed
a = 1
b = 2
c = 3
print(a < b < c)
print(a < b > 0)
print(c > b > a)
