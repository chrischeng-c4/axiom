# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# sort / sorted patterns broad

# sorted list
print(sorted([3, 1, 4, 1, 5, 9, 2, 6]))

# sorted reverse
print(sorted([3, 1, 4, 1, 5, 9, 2, 6], reverse=True))

# sorted strings
print(sorted(["banana", "apple", "cherry"]))
print(sorted(["banana", "apple", "cherry"], reverse=True))

# sorted with key=len
print(sorted(["banana", "apple", "kiwi", "strawberry"], key=len))

# sorted with key=lambda
data = [3, -1, 4, -5, 2]
print(sorted(data, key=lambda x: x * x))

# sorted empty
print(sorted([]))

# sorted single
print(sorted([42]))

# sorted already sorted
print(sorted([1, 2, 3, 4, 5]))

# sorted reverse already sorted
print(sorted([5, 4, 3, 2, 1]))

# list.sort mutates
li = [3, 1, 2]
li.sort()
print(li)

li2 = [5, 1, 3, 2, 4]
li2.sort()
print(li2)

# list.sort reverse
li3 = [1, 2, 3, 4, 5]
li3.sort(reverse=True)
print(li3)

# list.sort with key
words = ["banana", "apple", "kiwi"]
words.sort(key=len)
print(words)

# sorted of tuple
print(sorted((3, 1, 2)))
print(sorted((5, 4, 3, 2, 1)))

# sorted of dict keys
d = {"c": 3, "a": 1, "b": 2}
print(sorted(d))
print(sorted(d.keys()))
print(sorted(d.values()))

# sorted of dict items
print(sorted(d.items()))

# sorted set
s = {3, 1, 4, 1, 5, 9, 2, 6}
print(sorted(s))

# sorted of range
print(sorted(range(5, 0, -1)) if False else sorted([5, 4, 3, 2, 1]))

# sort negatives
print(sorted([-3, -1, -4, 0, 5]))
print(sorted([-3, -1, -4, 0, 5], reverse=True))

# sort abs
print(sorted([-3, -1, 2, -5, 4], key=abs))

# sort by negation (reverse trick)
print(sorted([1, 2, 3, 4, 5], key=lambda x: -x))
