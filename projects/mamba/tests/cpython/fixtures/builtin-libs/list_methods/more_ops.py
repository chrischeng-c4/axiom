# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
print([])
print([1])
print([1, 2, 3])
print(list())
print(list("abc"))
print(list(range(5)))
print(list((1, 2, 3)))

# append / extend / insert
a = [1, 2, 3]
a.append(4)
print(a)

a.extend([5, 6])
print(a)

a.insert(0, 0)
print(a)

a.insert(3, 99)
print(a)

# pop / remove
a = [1, 2, 3, 4]
print(a.pop())
print(a)

print(a.pop(0))
print(a)

a = [1, 2, 3, 2, 1]
a.remove(2)
print(a)

# reverse / sort
a = [3, 1, 4, 1, 5, 9, 2, 6]
a.reverse()
print(a)

a.sort()
print(a)

a.sort(reverse=True)
print(a)

words = ["banana", "apple", "cherry"]
words.sort(key=len)
print(words)

# slicing
a2 = [10, 20, 30, 40, 50]
print(a2[1:4])
print(a2[:2])
print(a2[3:])
print(a2[::2])
print(a2[::-1])
print(a2[-3:])
print(a2[:-2])

# sum/min/max/len on list
print(len([1, 2, 3]))
print(sum([1, 2, 3, 4, 5]))
print(min([3, 1, 4, 1, 5]))
print(max([3, 1, 4, 1, 5]))

# iteration
total = 0
for x in [1, 2, 3, 4, 5]:
    total += x
print(total)

# any/all
print(all([True, True, True]))
print(all([True, False]))
print(any([False, False, True]))
print(any([False, False]))
print(all([]))
print(any([]))

# repetition
print([0] * 5)
print([1, 2] * 3)

# concatenation
print([1, 2] + [3, 4])

# equality / comparison
print([1, 2, 3] == [1, 2, 3])
print([1, 2] < [1, 2, 0])
print([1, 2, 3] > [1, 2, 2])

# count/index
a3 = [1, 2, 3, 2, 1, 2]
print(a3.count(2))
print(a3.index(3))