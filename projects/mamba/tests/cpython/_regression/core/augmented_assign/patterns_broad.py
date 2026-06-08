# augmented assign patterns broad

# arithmetic
x = 10
x += 5
print(x)
x -= 3
print(x)
x *= 2
print(x)
x //= 3
print(x)
x %= 5
print(x)
x **= 3
print(x)

# with negatives
y = 100
y -= 200
print(y)
y += 50
print(y)
y //= -3
print(y)

# float
f = 1.0
f += 2.5
print(f)
f *= 2.0
print(f)

# bitwise
b = 0b1100
b &= 0b1010
print(b)
b |= 0b0001
print(b)
b ^= 0b1111
print(b)
b <<= 2
print(b)
b >>= 1
print(b)

# string
s = "hello"
s += " world"
print(s)
s *= 2
print(s)

# list
lst = [1, 2, 3]
lst += [4, 5]
print(lst)
lst *= 2
print(lst)

# in loop (accumulator)
total = 0
for i in range(5):
    total += i
print(total)

# in function
def count_evens(lst):
    count = 0
    for v in lst:
        if v % 2 == 0:
            count += 1
    return count

print(count_evens([1, 2, 3, 4, 5, 6]))

# dict value
d = {"a": 1}
d["a"] += 10
print(d["a"])

# list index
arr = [10, 20, 30]
arr[0] += 5
arr[2] *= 2
print(arr)
