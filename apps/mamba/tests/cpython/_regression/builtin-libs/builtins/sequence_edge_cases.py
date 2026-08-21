# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Edge cases for sequence / aggregate builtins

# any/all with generator expressions
print(any(x > 5 for x in [1, 2, 10]))
print(all(x > 0 for x in [1, 2, 3]))
print(any([]))
print(all([]))

# sum with start argument
print(sum([1, 2, 3]))
print(sum([1, 2, 3], 10))
print(sum([1.5, 2.5]))
print(sum([]))

# min/max with key + default
print(min([3, 1, 4, 1, 5]))
print(max([3, 1, 4, 1, 5]))
print(min(["bb", "aaa", "c"], key=len))
print(max([], default="empty"))

# sorted with key + reverse together
print(sorted([(1, 'z'), (2, 'a'), (3, 'm')], key=lambda p: p[1], reverse=True))
