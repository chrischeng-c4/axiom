# Conformance test: file-based module imports (#1190)
# Tests: import module, from module import, import as alias, module caching

# --- Basic import: variable access ---
import helper_module
print(helper_module.MODULE_VAR)
print(helper_module.GREETING)
print(helper_module.PI)

# --- from ... import variables ---
from helper_module import MODULE_VAR, GREETING, PI
print(MODULE_VAR)
print(GREETING)
print(PI)

# --- from ... import functions ---
from helper_module import add, multiply
print(add(3, 4))
print(multiply(5, 6))

# --- from ... import ... as alias ---
from helper_module import add as my_add
from helper_module import multiply as mul
print(my_add(10, 20))
print(mul(7, 8))

# --- import ... as alias: variable access ---
import helper_module as hm
print(hm.MODULE_VAR)
print(hm.GREETING)

# --- Module caching: second import returns same module ---
import helper_module
print(helper_module.MODULE_VAR)

print("all import tests passed")
