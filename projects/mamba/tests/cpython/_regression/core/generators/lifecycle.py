# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Generator lifecycle — close on del, pending finally (R11)

# close() on generator with pending finally
def gen_with_finally():
    try:
        yield 1
        yield 2
    finally:
        print('finally ran')

g = gen_with_finally()
next(g)  # suspend at first yield
g.close()  # should trigger finally

# del on active generator triggers close
def gen_with_cleanup():
    try:
        yield 1
    finally:
        print('finalized')

g2 = gen_with_cleanup()
next(g2)
del g2  # should trigger close -> finally
