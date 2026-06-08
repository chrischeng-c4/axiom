# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list insert/extend/sort broad

# insert at various positions
li = [1, 2, 3]
li.insert(0, 99)
print(li)

li2 = [1, 2, 3]
li2.insert(1, 99)
print(li2)

li3 = [1, 2, 3]
li3.insert(3, 99)
print(li3)

li4 = [1, 2, 3]
li4.insert(100, 99)  # append equivalent
print(li4)

li5 = [1, 2, 3]
li5.insert(-1, 99)  # insert before last
print(li5)

# extend with list
li = [1, 2]
li.extend([3, 4, 5])
print(li)

# extend with tuple
li = [1]
li.extend((2, 3, 4))
print(li)

# extend empty
li = [1, 2]
li.extend([])
print(li)

# append multiple
li = []
for x in [1, 2, 3, 4]:
    li.append(x)
print(li)

# sort basic
li = [3, 1, 4, 1, 5, 9, 2, 6]
li.sort()
print(li)

# sort reverse
li = [3, 1, 4, 1, 5]
li.sort(reverse=True)
print(li)

# sort empty
e = []
e.sort()
print(e)

# sort single
sing = [42]
sing.sort()
print(sing)
# sort of floats
fl = [3.14, 1.5, 2.7, 0.5]
fl.sort()
print(fl)

# reverse
li = [1, 2, 3, 4, 5]
li.reverse()
print(li)

# reverse on list-str method
ls = ["a", "b", "c", "d"]
ls.reverse()
print(ls)

# index
li = [10, 20, 30, 40]
print(li.index(20))
print(li.index(40))

# count
li = [1, 2, 3, 2, 2, 4]
print(li.count(2))
print(li.count(99))

# clear
li = [1, 2, 3]
li.clear()
print(li)
print(len(li))

# remove
li = [1, 2, 3, 2, 4]
li.remove(2)
print(li)