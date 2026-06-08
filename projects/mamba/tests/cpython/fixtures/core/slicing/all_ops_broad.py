# slicing broad

xs = [10, 20, 30, 40, 50, 60, 70, 80, 90]

# basic
print(xs[0:3])
print(xs[3:6])
print(xs[6:])
print(xs[:3])
print(xs[:])

# negative indices
print(xs[-3:])
print(xs[:-3])
print(xs[-3:-1])
print(xs[-1:])

# step
print(xs[::2])
print(xs[::3])
print(xs[1::2])
print(xs[1:7:2])

# reverse
print(xs[::-1])
print(xs[::-2])
print(xs[7:2:-1])
print(xs[7:2:-2])

# out-of-bound slices (no error)
print(xs[100:200])
print(xs[-100:-50])
print(xs[:100])
print(xs[-100:])

# empty
print(xs[3:3])
print(xs[5:2])

# string slicing
s = "abcdefg"
print(s[0:3])
print(s[::-1])
print(s[1:-1])
print(s[::2])

# tuple slicing
t = (1, 2, 3, 4, 5)
print(t[1:3])
print(t[-2:])

# copy via full slice
lst = [1, 2, 3]
copy = lst[:]
copy.append(4)
print(lst)
print(copy)
