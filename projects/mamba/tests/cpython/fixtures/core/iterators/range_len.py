# Regression: len(range(a, b, c)) must answer in O(1) without consuming
# the iterator. The multi-arg range emits an iterator handle (tagged
# int), and mb_len previously returned 0 for any int it saw.

print(len(range(5, 20, 3)))
print(len(range(10)))
print(len(range(0, 100, 10)))
print(len(range(10, 0, -1)))
print(len(range(5, 5)))
print(len(range(0, -10, -2)))
print(len(range(0, 10, 1)))
print(len(range(-5, 5)))
