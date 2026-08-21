# import deep broad

# module as
import math as m
print(m.pi)
print(m.floor(3.7))

# from X import A, B, C
from math import pi, floor, ceil
print(pi)
print(floor(3.7))
print(ceil(3.2))

# from X import * (might be limited — test with only specific module exposures)
# Leaving out - usually behaves differently

# nested call into imported
from math import sqrt
print(sqrt(16))
print(sqrt(25))

# multiple modules
import math
import json
print(math.pi)
print(json.dumps(42))

# rename with as
from math import pi as PI_VAL
print(PI_VAL)

# from X import Y as Z
from math import sqrt as square_root
print(square_root(9))
print(square_root(36))

# import + dotted access
import json
d = {"a": 1, "b": 2}
s = json.dumps(d, sort_keys=True)
print(s)

# reuse after reimport
import math as mm
print(mm.pi == math.pi)
