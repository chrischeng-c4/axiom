# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list mutation patterns broad

# append
li = []
li.append(1)
li.append(2)
li.append(3)
print(li)

# extend
li = [1, 2]
li.extend([3, 4, 5])
print(li)

li.extend([6])
print(li)

li.extend([])
print(li)

# insert
li = [1, 2, 4, 5]
li.insert(2, 3)
print(li)

li.insert(0, 0)
print(li)

li.insert(100, 99)  # beyond end
print(li)

# pop (default last)
li = [1, 2, 3, 4, 5]
print(li.pop())
print(li)
print(li.pop())
print(li)

# pop specific idx
li = [10, 20, 30, 40]
print(li.pop(0))
print(li)
print(li.pop(1))
print(li)

# remove first occurrence
li = [1, 2, 3, 2, 1]
li.remove(2)
print(li)
li.remove(1)
print(li)

# clear
li = [1, 2, 3, 4, 5]
li.clear()
print(li)
print(len(li))

# reverse
li = [1, 2, 3, 4, 5]
li.reverse()
print(li)

li = ["a", "b", "c"]
li.reverse()
print(li)

# sort
li = [3, 1, 4, 1, 5, 9, 2, 6]
li.sort()
print(li)

# index
li = [10, 20, 30, 20, 10]
print(li.index(20))
print(li.index(10))
print(li.index(30))

# count
print(li.count(10))
print(li.count(20))
print(li.count(99))

# copy
li = [1, 2, 3]
c = li.copy()
c.append(4)
print(li)
print(c)

# list() constructor
print(list([1, 2, 3]))
print(list((4, 5, 6)))
print(list("abc"))
print(list(range(5)))
print(list())

# + concatenation
print([1, 2] + [3, 4])
print([1] + [2] + [3])

# * repetition
print([0] * 5)
print([1, 2] * 3)
print([] * 5)

# in / not in
li = [1, 2, 3]
print(2 in li)
print(99 in li)
print(99 not in li)

# nested append
outer = []
for i in range(3):
    inner = []
    for j in range(3):
        inner.append(i * j)
    outer.append(inner)
for row in outer:
    print(row)