# slicing with steps broad

# basic step
print([1, 2, 3, 4, 5, 6, 7, 8, 9, 10][::2])
print([1, 2, 3, 4, 5, 6, 7, 8, 9, 10][1::2])
print([1, 2, 3, 4, 5, 6, 7, 8, 9, 10][::3])

# explicit start/stop/step
li = list(range(20))
print(li[0:10:2])
print(li[5:15:3])
print(li[1:10:2])
print(li[0:20:5])

# negative step (reverse)
print([1, 2, 3, 4, 5][::-1])
print("abcde"[::-1])
print("hello"[::-1])
print(list(range(10))[::-1])

# negative step with explicit range (forward direction)
# skip: tricky cases

# step larger than length
print([1, 2, 3][::10])
print([1, 2, 3][::100])

# step on string
print("abcdefg"[::2])
print("abcdefg"[1::2])

# slice assignment - skip, mutation

# slice returns new list
a = [1, 2, 3, 4, 5]
b = a[:]
b[0] = 99
print(a)
print(b)

# slice with negative indices
print([10, 20, 30, 40, 50][-3:])
print([10, 20, 30, 40, 50][:-2])
print([10, 20, 30, 40, 50][-4:-1])

# string slice with negative
print("abcdefg"[-3:])
print("abcdefg"[:-2])
print("abcdefg"[-5:-1])

# tuple slice with step
print((1, 2, 3, 4, 5)[::2])

# empty slices
print([1, 2, 3][10:20])
print([1, 2, 3][2:1])
print([1, 2, 3][5:5])
print([1, 2, 3][0:0])

# full slice
print([1, 2, 3][:])
print([1, 2, 3][::])

# length-preserving slice
li = list(range(10))
print(li[:])

# slice from end
print([1, 2, 3, 4, 5][-1:])
print([1, 2, 3, 4, 5][:1])
