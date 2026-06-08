# RUN: parse

# Extended star assignment
first, *rest = [1, 2, 3, 4, 5]
*init, last = [1, 2, 3, 4, 5]
head, *middle, tail = [1, 2, 3, 4, 5]

# Nested unpacking
(a, b), c = (1, 2), 3
a, (b, c) = 1, (2, 3)
