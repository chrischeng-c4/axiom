# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list access patterns broad

lst = [10, 20, 30, 40, 50]

# basic indexing
print(lst[0])
print(lst[1])
print(lst[4])

# negative indexing
print(lst[-1])
print(lst[-2])
print(lst[-5])

# slicing basic
print(lst[:])
print(lst[0:])
print(lst[:5])
print(lst[0:5])
print(lst[1:4])
print(lst[2:3])

# slicing with step
print(lst[::2])
print(lst[1::2])
print(lst[::-1])
print(lst[::-2])

# slicing with negative indices
print(lst[-3:])
print(lst[:-2])
print(lst[-4:-1])

# out-of-bound slicing (silent)
print(lst[:100])
print(lst[100:])
print(lst[-100:])

# empty slice
print(lst[3:2])
print(lst[5:])

# len
print(len(lst))
print(len([]))
print(len([1]))

# in / not in
print(10 in lst)
print(99 in lst)
print(99 not in lst)

# concatenation
print([1, 2] + [3, 4])
print([] + [1])
print([1] + [])

# repetition
print([0] * 5)
print([1, 2] * 3)
print([] * 10)

# bool
print(bool([]))
print(bool([0]))
print(bool([0, 0]))

# iteration
total = 0
for v in lst:
    total += v
print(total)

# enumerate
for i, v in enumerate(lst):
    print(i, v)