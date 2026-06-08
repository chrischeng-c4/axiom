# Regression: `b = a` must copy the value for immutables, not alias a's VReg.
# MIR lowering's first-assignment path reused the source VReg directly, so
# a later `b = ...` overwrote a as well:
#
#   a = 5
#   b = a
#   b = 10
#   print(a, b)   # was "10 10", should be "5 10"
#
# Fixed by always allocating a fresh VReg + Copy in the define-new-symbol
# branch — matching the HirStmt::Let path.

a = 5
b = a
print(a, b)
b = 10
print(a, b)

# Same with strings
s1 = "hello"
s2 = s1
print(s1, s2)
s2 = "world"
print(s1, s2)

# Chained via multiple hops
x = 1
y = x
z = y
z = 99
print(x, y, z)

# Mutable aliasing (still shared — this is correct Python behavior)
p = [1, 2, 3]
q = p
q.append(4)
print(p)  # shared reference: [1, 2, 3, 4]

# After rebinding q, p is unchanged
q = [99]
print(p)  # still [1, 2, 3, 4]
print(q)
