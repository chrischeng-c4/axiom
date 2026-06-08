# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# sorted/min/max with key broad

# sorted basic
print(sorted([3, 1, 4, 1, 5, 9, 2, 6]))
print(sorted([3, 1, 4]))

# sorted reverse
print(sorted([3, 1, 4, 1, 5], reverse=True))
print(sorted([1, 2, 3], reverse=True))

# sorted with key
words = ["bananas", "apple", "cherries", "date"]
print(sorted(words))
print(sorted(words, key=len))

# sorted by negative
nums = [3, 1, 4, 1, 5, 9, 2, 6]
print(sorted(nums, key=lambda x: -x))

# sorted tuples by idx
pairs = [(1, "c"), (2, "a"), (3, "b")]
print(sorted(pairs, key=lambda p: p[1]))

# sorted strings
print(sorted("banana"))
print(sorted(["python", "ruby", "go", "rust"]))

# sorted dict items
d = {"b": 2, "a": 1, "c": 3}
print(sorted(d.items()))
print(sorted(d.items(), key=lambda kv: kv[1]))

# sorted with key + reverse (unique lens to avoid tie-stability)
print(sorted(words, key=len, reverse=True))

# min with key
print(min([1, 2, 3]))
print(min([3, 1, 2]))
print(min(words, key=len))
print(min([-1, -2, -3], key=abs))

# max with key
print(max([1, 2, 3]))
print(max(words, key=len))
print(max([-1, -2, -3], key=abs))

# min/max with default
print(min([], default=999))
print(max([], default=-1))

# min/max multiple args
print(min(3, 1, 2))
print(max(3, 1, 2))

# sort list in place
li = [3, 1, 4, 1, 5]
li.sort()
print(li)
li.sort(reverse=True)
print(li)

# sort list in place with key
w = ["pear", "kiwis", "banana"]
w.sort(key=len)
print(w)

# sorted set
print(sorted({3, 1, 4, 1, 5}))
