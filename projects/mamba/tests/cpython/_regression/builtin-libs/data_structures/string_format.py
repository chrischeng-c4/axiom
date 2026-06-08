# String format: positional args and f-string format specs
print('Hello, {}!'.format('world'))
print('{0} + {1} = {2}'.format(1, 2, 3))
print(f'{3.14:.1f}')
print(f'{"hello":>10}')
# String format: keyword argument substitution via str.format()
# String format: keyword argument substitution
print('{name} is {age}'.format(name='Bob', age=25))
