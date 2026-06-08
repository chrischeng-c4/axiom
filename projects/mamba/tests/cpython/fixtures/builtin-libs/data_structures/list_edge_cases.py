# List edge cases: empty list, out-of-range slice, negative index, nested list
a = []
print(len(a))
print(a[::-1])
# Out-of-range slicing (clamped)
a = [10, 20, 30]
print(a[5:10])
print(a[-10:2])
# Nested list indexing
print([[1, 2], [3, 4]][0][1])
# List edge cases: exception handling for pop and index on empty/missing
try:
    [].pop()
except IndexError:
    print('caught')
try:
    [1, 2].index(99)
except ValueError:
    print('caught')
