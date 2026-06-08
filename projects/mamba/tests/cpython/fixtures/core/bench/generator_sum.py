# tier: exploratory
# category: workload
# inclusion_reason: generator iteration over 1M items (tracking #1260)
def gen(n: int):
    i: int = 0
    while i < n:
        yield i
        i = i + 1

total: int = 0
for x in gen(1000000):
    total = total + x
print(total)
