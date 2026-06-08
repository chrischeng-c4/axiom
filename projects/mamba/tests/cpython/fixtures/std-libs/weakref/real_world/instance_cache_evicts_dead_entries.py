# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "real_world"
# case = "instance_cache_evicts_dead_entries"
# subject = "weakref.WeakValueDictionary"
# kind = "semantic"
# xfail = "mamba refcount-only: WeakValueDictionary cache does not evict collected values (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: a WeakValueDictionary-backed instance cache returns the cached object while referenced and evicts it once the last strong reference is dropped and collected"""
import gc
import weakref


# A common real-world idiom: an interning/identity cache that hands out a
# single shared instance per key without keeping those instances alive
# itself. The WeakValueDictionary holds only weak references, so an entry
# vanishes once the application drops its last strong reference.
class Resource:
    def __init__(self, key):
        self.key = key


class ResourceCache:
    def __init__(self):
        self._cache = weakref.WeakValueDictionary()

    def get(self, key):
        obj = self._cache.get(key)
        if obj is None:
            obj = Resource(key)
            self._cache[key] = obj
        return obj


cache = ResourceCache()

# First lookup creates and caches; a second lookup returns the same object.
a = cache.get("config")
b = cache.get("config")
assert a is b, "cache returns the identical shared instance"
assert len(cache._cache) == 1, f"one live entry = {len(cache._cache)!r}"

# Drop both strong references; the cache must evict the dead entry.
del a
del b
gc.collect()
assert len(cache._cache) == 0, f"cache evicted dead entry = {len(cache._cache)!r}"

# A fresh lookup after eviction builds a brand-new instance.
c = cache.get("config")
assert c.key == "config", f"rebuilt instance key = {c.key!r}"

print("instance_cache_evicts_dead_entries OK")
