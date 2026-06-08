# RUN: parse
# CPython 3.12 test_list: list operations

# Construction
lst = []
lst = [1, 2, 3]
lst = list()
lst = list(range(5))
lst = list("hello")

# Indexing and slicing
lst = [1, 2, 3, 4, 5]
x = lst[0]
x = lst[-1]
x = lst[1:3]
x = lst[:3]
x = lst[2:]
x = lst[::2]
x = lst[::-1]

# Modification
lst[0] = 10
lst[1:3] = [20, 30]
del lst[0]
del lst[1:3]

# Methods (parse only, not execute)
lst = [3, 1, 2]
lst.append(4)
lst.extend([5, 6])
lst.insert(0, 0)
lst.remove(3)
lst.pop()
lst.pop(0)
lst.sort()
lst.sort(key=lambda x: -x, reverse=True)
lst.reverse()
lst.clear()
idx = lst.index(2)
cnt = lst.count(1)
lst2 = lst.copy()

# Operators
lst = [1, 2] + [3, 4]
lst = [0] * 5
b = 1 in lst
b = 10 not in lst

# List comprehensions
squares = [x**2 for x in range(10)]
evens = [x for x in range(10) if x % 2 == 0]
matrix = [[i * j for j in range(5)] for i in range(5)]
flat = [x for row in matrix for x in row]

# Nested lists
nested = [[1, 2], [3, 4], [5, 6]]
x = nested[0][1]

# len, min, max, sum
n = len(lst)
mn = min(lst)
mx = max(lst)
s = sum(lst)

# Sorting
lst = sorted([3, 1, 2])
lst = sorted([3, 1, 2], reverse=True)
