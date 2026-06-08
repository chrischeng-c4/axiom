# slice object semantics

# basic slicing
lst = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(lst[2:5])
print(lst[:5])
print(lst[5:])
print(lst[:])
print(lst[::2])
print(lst[1::2])
print(lst[::3])
print(lst[::-1])
print(lst[::-2])
print(lst[-3:])
print(lst[:-3])
print(lst[-5:-2])

# out of bounds, empty
print(lst[100:200])
print(lst[5:2])
print(lst[-100:2])
print(lst[5:100])

# string slicing
s = "abcdefghij"
print(s[0:3])
print(s[3:])
print(s[:3])
print(s[::-1])
print(s[1:8:2])

# tuple slicing
t = (0, 1, 2, 3, 4)
print(t[1:4])
print(t[::2])

# negative steps in slice assignment not tested (has known issues)

# empty slices
print(lst[3:3])
print(s[2:2])
