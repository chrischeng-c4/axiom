# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# any/all/sum/abs/round deep broad

# any
print(any([True, False, False]))
print(any([False, False, True]))
print(any([False, False, False]))
print(any([]))
print(any([0, 0, 0]))
print(any([0, 1, 0]))
print(any([1, 0, 0]))

# any w/ generator
print(any(x > 5 for x in [1, 2, 3]))
print(any(x > 5 for x in [1, 5, 10]))
print(any(x > 5 for x in []))

# any w/ strings
print(any(["", "", "a"]))
print(any(["", "", ""]))
print(any(["x"]))

# all
print(all([True, True, True]))
print(all([True, False, True]))
print(all([]))
print(all([1, 1, 1]))
print(all([1, 0, 1]))
print(all([0, 1, 1]))

# all w/ generator
print(all(x > 0 for x in [1, 2, 3]))
print(all(x > 0 for x in [1, 0, 3]))
print(all(x > 0 for x in []))

# sum
print(sum([1, 2, 3, 4, 5]))
print(sum([]))
print(sum([10]))
print(sum([-1, -2, -3]))
print(sum([1.5, 2.5]))
print(sum([1, 2, 3], 100))

# sum w/ generator
print(sum(x * 2 for x in [1, 2, 3]))
print(sum(x for x in range(10)))
print(sum(x for x in range(100)))

# abs
print(abs(5))
print(abs(-5))
print(abs(0))
print(abs(1.5))
print(abs(-1.5))

# round no digits
print(round(3.5))
print(round(2.5))
print(round(-3.5))
print(round(0.0))

# round with digits
print(round(3.14159, 2))
print(round(3.14159, 4))

# min/max basic
print(min([3, 1, 2]))
print(max([3, 1, 2]))
print(min([1]))
print(max([1]))
print(min(3, 1, 2))
print(max(3, 1, 2))

# min/max with strings
print(min(["banana", "apple", "cherry"]))
print(max(["banana", "apple", "cherry"]))

# min/max with key
print(min([-3, -1, -2], key=abs))
print(max([-3, -1, -2], key=abs))
