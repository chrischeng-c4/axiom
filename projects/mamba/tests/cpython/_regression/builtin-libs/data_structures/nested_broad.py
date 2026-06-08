# nested data structures broad

# list of lists (2D)
grid = [[0, 1, 2], [3, 4, 5], [6, 7, 8]]
print(grid[0])
print(grid[1][1])
print(grid[2][2])

# sum of row
print(sum(grid[0]))
print(sum(grid[1]))

# mutation
grid[1][1] = 99
print(grid[1])

# nested append
col0 = []
for row in grid:
    col0.append(row[0])
print(col0)

# flat from 2D
flat = []
for row in grid:
    for v in row:
        flat.append(v)
print(flat)

# list of dicts
people = [
    {"name": "alice", "age": 30},
    {"name": "bob", "age": 25},
    {"name": "carol", "age": 35},
]
for p in people:
    print(p["name"], p["age"])

# sort list of dicts
people_sorted = sorted(people, key=lambda p: p["age"])
for p in people_sorted:
    print(p["name"])

# dict of lists
scores = {"alice": [95, 87, 92], "bob": [70, 80, 75]}
print(scores["alice"])
print(sum(scores["alice"]))
print(sum(scores["bob"]))

# dict of dicts
matrix = {
    "row0": {"col0": 1, "col1": 2},
    "row1": {"col0": 3, "col1": 4},
}
print(matrix["row0"]["col0"])
print(matrix["row1"]["col1"])

# iter nested dict
for rkey in sorted(matrix):
    for ckey in sorted(matrix[rkey]):
        print(rkey, ckey, matrix[rkey][ckey])

# list of tuples
items = [("a", 1), ("b", 2), ("c", 3)]
for k, v in items:
    print(k, v)

# tuple of lists
t = ([1, 2, 3], [4, 5, 6])
print(t[0])
print(t[1])
print(sum(t[0]) + sum(t[1]))

# nested list comp
multi = [[x * y for y in range(3)] for x in range(3)]
for row in multi:
    print(row)
