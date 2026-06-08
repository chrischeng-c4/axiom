# Assert statement

# Passing assertions
assert True
assert 1 == 1
assert len([1, 2, 3]) == 3
print("basic asserts passed")

# Assert with message — caught
try:
    assert False, "expected failure"
except AssertionError as e:
    print("caught assertion")

# Assert that passes with message (message not used)
assert 2 + 2 == 4, "math is broken"
print("math assert passed")

# Assert in function
def check_positive(n):
    assert n > 0, "not positive"
    return n * 2

print(check_positive(5))

try:
    check_positive(-1)
except AssertionError as e:
    print("caught negative")
