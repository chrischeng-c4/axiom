# Pattern matching edge cases

# Mapping pattern with capture
def test_mapping(d):
    match d:
        case {'action': 'move', 'x': x, 'y': y}:
            print(f'move to {x},{y}')
        case _:
            print('unknown')

test_mapping({'action': 'move', 'x': 10, 'y': 20})

# OR pattern
def test_or(val):
    match val:
        case 1 | 2 | 3:
            print('small')
        case x if x > 40:
            print(f'big: {x}')
        case _:
            print('other')

test_or(42)

# Sequence pattern with star
def test_seq(seq):
    match seq:
        case [first, *rest]:
            print(f'first={first}, rest={rest}')

test_seq([1, 2, 3, 4, 5])

# Nested pattern: mapping containing sequence of dicts
def test_nested(data):
    match data:
        case {'users': [{'name': first_name}, *_]}:
            print(f'first user: {first_name}')
        case _:
            print('no match')

test_nested({'users': [{'name': 'Alice'}, {'name': 'Bob'}]})
