# nested data access broad

# list of lists
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
print(matrix[0])
print(matrix[1])
print(matrix[2])
print(matrix[0][0])
print(matrix[1][1])
print(matrix[2][2])
print(matrix[0][2])
print(matrix[2][0])

# len
print(len(matrix))
print(len(matrix[0]))

# iterate nested
for row in matrix:
    for x in row:
        print(x)

# sum of row
for row in matrix:
    total = 0
    for x in row:
        total += x
    print(total)

# sum of column
for col in range(3):
    total = 0
    for row in range(3):
        total += matrix[row][col]
    print(total)

# dict of dicts
users = {
    "alice": {"age": 30, "city": "NYC"},
    "bob": {"age": 25, "city": "LA"},
}

print(users["alice"]["age"])
print(users["bob"]["city"])
print(users["alice"]["city"])
print(users["bob"]["age"])

# list of dicts
items = [
    {"name": "a", "val": 1},
    {"name": "b", "val": 2},
    {"name": "c", "val": 3},
]

for item in items:
    print(item["name"], item["val"])

# sum values from list of dicts
total = 0
for item in items:
    total += item["val"]
print(total)

# dict of lists
groups = {"odd": [1, 3, 5, 7], "even": [2, 4, 6, 8]}
print(groups["odd"])
print(groups["even"])
print(groups["odd"][0])
print(groups["even"][-1])
print(len(groups["odd"]))

# modify nested
m2 = [[0, 0, 0], [0, 0, 0], [0, 0, 0]]
m2[0][0] = 1
m2[1][1] = 5
m2[2][2] = 9
for row in m2:
    print(row)

# nested method calls
lists = [[3, 1, 2], [6, 4, 5], [9, 7, 8]]
for li in lists:
    li.sort()
for li in lists:
    print(li)

# deeply nested
d3 = {"a": {"b": {"c": 42}}}
print(d3["a"]["b"]["c"])

# mixed nested
data = {"numbers": [1, 2, 3], "nested": {"key": "value"}}
print(data["numbers"][0])
print(data["nested"]["key"])
print(len(data["numbers"]))
