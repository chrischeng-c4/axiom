# star unpacking ops broad

# multi-star assignment
first, *rest = [1, 2, 3, 4, 5]
print(first)
print(rest)

*init, last = [1, 2, 3, 4, 5]
print(init)
print(last)

head, *mid, tail = [1, 2, 3, 4, 5]
print(head)
print(mid)
print(tail)

# empty rest
x, *e = [1]
print(x)
print(e)

# with tuple rhs
p, *q = (10, 20, 30, 40)
print(p)
print(q)

*before, final = [7, 8, 9]
print(before)
print(final)

# single element
only, *empty = [42]
print(only)
print(empty)

# all rest
*all_rest, = [1, 2, 3]
print(all_rest)

# head+middle+tail with short
first, *middle, last = [1, 2]
print(first)
print(middle)
print(last)
