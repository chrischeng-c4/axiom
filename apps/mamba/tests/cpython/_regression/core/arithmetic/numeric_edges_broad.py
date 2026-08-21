# numeric edge cases

# integer ops
print(3 ** 100)  # big int
print(2 ** 64)
print(10 // 3)
print(-10 // 3)
print(10 // -3)
print(-10 // -3)

# float precision
print(round(0.1 + 0.2, 10))
print(round(1.0 / 3, 10))

# int overflow safely
big = 10 ** 18
print(big)
print(big + 1)
print(big * 2)

# int from float
print(int(3.999))
print(int(-3.999))
print(int(0.5))
print(int(-0.5))

# float from int
print(float(42))
print(float(-42))
print(float(0))

# bool arithmetic
print(True + True)
print(True + False)
print(True * 5)
print(sum([1, 0, 1, 1]))

# bool conversion
print(bool(-0.0))
print(bool(1e-300))
print(bool([1]))
print(bool((1,)))
print(bool({1}))
print(bool({"a": 1}))

# abs / sign
print(abs(-3.14))
print(abs(0))
print(abs(-0.0))

# min/max with key
print(min([-3, 1, -2], key=abs))
print(max([-3, 1, -2], key=abs))

# sum with start
print(sum([1, 2, 3]))
print(sum([1, 2, 3], 10))
