values = iter([10, 20, 30])
while True:
    try:
        x = next(values)
    except StopIteration:
        break
    print(x)
