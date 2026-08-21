# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Collection builtin edge cases: sorted with key/reverse, sum with start
# Collection builtin edge cases
print(sorted([3, 1, 4, 1, 5]))
print(sorted('hello'))
print(sorted([3, 1, 2], reverse=True))
print(sorted(['banana', 'apple'], key=len))
print(all([True, True, True]))
print(all([True, False, True]))
print(any([False, False, True]))
print(any([]))
print(all([]))
print(min([3, 1, 4], key=lambda x: -x))
print(max([], default='empty'))
print(min('hello'))
print(sum([1.5, 2.5], start=10))
