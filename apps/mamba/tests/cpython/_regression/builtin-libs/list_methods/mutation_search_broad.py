# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list methods deep broad

# clear
li = [1, 2, 3, 4]
li.clear()
print(li)

# reverse in place
li2 = [1, 2, 3, 4, 5]
li2.reverse()
print(li2)

# sort in place, default + key + reverse
li3 = [3, 1, 4, 1, 5, 9, 2, 6]
li3.sort()
print(li3)

li4 = [3, 1, 4, 1, 5, 9, 2, 6]
li4.sort(reverse=True)
print(li4)

# sort strings by length
words = ["apple", "fig", "banana", "ok"]
words.sort(key=len)
print(words)

# sort with nested list key
people = [["bob", 30], ["alice", 25], ["carl", 35]]
people.sort(key=lambda p: p[1])
print(people)

# sorted (copy)
orig = [3, 1, 2]
copy_s = sorted(orig)
print(orig)
print(copy_s)

# extend + concat
a = [1, 2]
b = [3, 4]
a.extend(b)
print(a)
print(a + [5, 6])

# insert
c = [10, 20, 30]
c.insert(1, 15)
print(c)
c.insert(0, 5)
print(c)
c.insert(100, 999)  # beyond end: append
print(c)

# pop default + specific idx
d = [1, 2, 3, 4, 5]
print(d.pop())
print(d)
print(d.pop(0))
print(d)
print(d.pop(1))
print(d)

# index (default only)
e = [10, 20, 30, 20, 50]
print(e.index(20))
print(e.index(50))

# count
print([1, 2, 3, 1, 1, 2].count(1))
print([1, 2, 3].count(99))

# * multiplication
print([0] * 5)
print([1, 2] * 3)
print([1, 2] * 0)

# contains
print(3 in [1, 2, 3])
print(99 in [1, 2, 3])
print(3 not in [1, 2, 3])

# list() conversion
print(list("abc"))
print(list(range(5)))
print(list((1, 2, 3)))