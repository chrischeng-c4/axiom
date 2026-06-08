# RUN: parse
# CPython 3.12 test_string: string slicing and indexing

s = "Hello, World!"

# Indexing
first = s[0]
last = s[-1]

# Basic slicing
hello = s[0:5]
world = s[7:12]

# Omitted bounds
from_start = s[:5]
to_end = s[7:]
full = s[:]

# Negative indices
last_five = s[-5:]
without_last = s[:-1]

# Step slicing
every_other = s[::2]
reversed_s = s[::-1]
step_neg = s[10:0:-2]

# Empty slices
empty = s[5:5]
empty2 = s[10:5]
