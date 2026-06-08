# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator-based context manager pattern (R10) — both normal and
# exception paths.

# Normal path: acquire -> body -> release
def managed_resource():
    print('acquire')
    try:
        yield 'resource'
    except ValueError as e:
        print('caught:', e)
    finally:
        print('release')

g = managed_resource()
resource = next(g)
print('using:', resource)
try:
    next(g)
except StopIteration:
    pass

# Exception path: acquire -> body raises -> generator catches -> release
g2 = managed_resource()
resource = next(g2)
print('using:', resource)
try:
    g2.throw(ValueError('boom'))
except StopIteration:
    pass
