# Iterator composition with generators — exercises the four canonical
# composition adapters (enumerate, zip, map, filter) plus a chained
# composition (enumerate(zip(...))) so we cover both the
# adapter-of-generator and generator-of-adapter directions.

def gen():
    yield 'a'
    yield 'b'
    yield 'c'

# enumerate over a generator
print(list(enumerate(gen())))

# zip across two generators
def gn():
    yield 1
    yield 2
    yield 3

print(list(zip(gn(), gen())))

# map on a generator
def gm():
    yield 1
    yield 2
    yield 3

print(list(map(lambda x: x * 2, gm())))

# filter on a generator
def gf():
    yield 1
    yield 2
    yield 3
    yield 4

print(list(filter(lambda x: x % 2 == 0, gf())))

# chained composition: enumerate(zip(gen, gen))
def ga():
    yield 'a'
    yield 'b'

def gb():
    yield 1
    yield 2

print(list(enumerate(zip(ga(), gb()))))
