# Augmented assignment: list concatenation and slice ops (from CPython test_augassign.testSequences)
x = [1, 2]
x += [3, 4]
x *= 2
print(x)

x = [1, 2, 3]
y = x
x[1:2] *= 2
y[1:2] += [1]
print(x)
print(x is y)
