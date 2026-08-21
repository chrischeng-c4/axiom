# Protocol dunders on builtin dict and list — direct method calls.
# Mamba previously raised AttributeError; now they dispatch through
# the same paths as the operator forms.

# ── dict ──
d = {1: 'a', 2: 'b'}
print(d.__getitem__(1))           # a
print(d.__contains__(1))          # True
print(d.__contains__(99))         # False
print(d.__len__())                # 2

d.__setitem__(3, 'c')
print(d.__getitem__(3))           # c
d.__delitem__(3)
print(d.__contains__(3))          # False

print(d.__or__({4: 'd'}))         # {1: 'a', 2: 'b', 4: 'd'}
print({0: 'z'}.__ror__(d))        # {1: 'a', 2: 'b', 0: 'z'}

print(d.__eq__({1: 'a', 2: 'b'})) # True
print(d.__eq__({1: 'a'}))         # False
print(d.__ne__({}))               # True
print(d.__ne__({1: 'a', 2: 'b'})) # False

# ── list ──
L = [10, 20, 30]
print(L.__getitem__(1))           # 20
print(L.__contains__(20))         # True
print(L.__contains__(99))         # False
print(L.__len__())                # 3

L.__setitem__(0, 99)
print(L.__getitem__(0))           # 99
L.__delitem__(0)
print(L)                          # [20, 30]

print(L.__add__([40, 50]))        # [20, 30, 40, 50]
print(L.__mul__(2))               # [20, 30, 20, 30]
print(L.__rmul__(3))              # [20, 30, 20, 30, 20, 30]

print(L.__eq__([20, 30]))         # True
print(L.__eq__([20]))             # False
print(L.__ne__([1]))              # True
print(L.__ne__([20, 30]))         # False
