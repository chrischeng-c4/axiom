# import variations

import math
print(math.pi)
print(math.sqrt(16))

import math as m
print(m.pi)
print(m.sqrt(25))

from math import pi, e
print(pi)
print(e)

from math import sqrt
print(sqrt(36))

from math import sqrt as square_root
print(square_root(49))

# multi-import
from math import floor, ceil
print(floor(3.7))
print(ceil(3.2))

# import json basic
import json
print(json.dumps({"a": 1}))
print(json.loads("[1, 2, 3]"))

# functools
from functools import reduce
print(reduce(lambda a, b: a + b, [1, 2, 3, 4]))

# itertools
from itertools import chain
print(list(chain([1, 2], [3, 4])))

from itertools import islice
print(list(islice(range(10), 3)))
