# slice step patterns broad

lst = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# positive step
print(lst[::2])
print(lst[1::2])
print(lst[::3])
print(lst[1::3])
print(lst[2:8:2])

# negative step
print(lst[::-1])
print(lst[::-2])
print(lst[8:0:-1])
print(lst[8:0:-2])
print(lst[-1:-5:-1])

# full form
print(lst[0:10:1])
print(lst[0:10:2])

# step bigger than length
print(lst[::100])

# string slicing
s = "abcdefghij"
print(s[::2])
print(s[::-1])
print(s[1:9:2])

# tuple slicing
t = (10, 20, 30, 40, 50)
print(t[::2])
print(t[::-1])
print(t[1::2])

# bytes slicing
bs = b"abcdefgh"
print(bs[::2])
print(bs[::-1])
