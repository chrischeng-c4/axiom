# Del statement: dict and list subscript deletion

# Del from dict
d = {"a": 1, "b": 2, "c": 3}
del d["b"]
print(d)

# Del from list
lst = [10, 20, 30, 40]
del lst[1]
print(lst)

# Del multiple from dict
d2 = {"x": 1, "y": 2, "z": 3}
del d2["x"]
del d2["z"]
print(d2)

# Del last element from list
lst2 = [1, 2, 3]
del lst2[2]
print(lst2)

# Del with negative index
lst3 = [10, 20, 30]
del lst3[-1]
print(lst3)
