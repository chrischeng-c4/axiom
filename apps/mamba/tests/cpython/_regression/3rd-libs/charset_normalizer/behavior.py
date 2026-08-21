"""Behavior contract for third-party charset_normalizer package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import charset_normalizer  # type: ignore[import]
import io

# Rule 1: from_bytes detects encoding for ASCII-safe UTF-8
_result1 = charset_normalizer.from_bytes(b"Hello, World!")
_best1 = _result1.best()
assert _best1 is not None, "ASCII text has a best match"
assert "utf" in _best1.encoding.lower() or "ascii" in _best1.encoding.lower(), \
    f"ASCII detected as utf/ascii: {_best1.encoding!r}"

# Rule 2: detect() returns dict with encoding, confidence, language
_det2 = charset_normalizer.detect(b"Hello World")
assert isinstance(_det2, dict), f"detect type = {type(_det2)!r}"
assert "encoding" in _det2, "encoding key"
assert "confidence" in _det2, "confidence key"
assert "language" in _det2, "language key"
assert _det2["encoding"] is not None, "encoding is not None"
assert isinstance(_det2["confidence"], float), \
    f"confidence is float: {type(_det2['confidence'])!r}"
assert 0.0 <= _det2["confidence"] <= 1.0, f"confidence in [0,1]: {_det2['confidence']!r}"

# Rule 3: from_bytes returns CharsetMatches iterable
_result3 = charset_normalizer.from_bytes(b"test bytes 12345")
_list3 = list(_result3)
assert isinstance(_list3, list), f"iteration type = {type(_list3)!r}"
# Can be empty for undetectable content

# Rule 4: is_binary returns False for text, True for binary data
assert charset_normalizer.is_binary(b"Hello text") is False, "text not binary"
assert charset_normalizer.is_binary(b"\x00\x01\x02\x03" * 50) is True, "null bytes binary"

# Rule 5: from_fp reads from file-like object
_data5 = "English text sample for detection.".encode("utf-8")
_fp5 = io.BytesIO(_data5)
_result5 = charset_normalizer.from_fp(_fp5)
_best5 = _result5.best()
assert _best5 is not None, "fp result has best match"

# Rule 6: CharsetMatch has encoding attribute as string
_result6 = charset_normalizer.from_bytes("résumé café".encode("utf-8"))
_best6 = _result6.best()
assert _best6 is not None, "utf-8 text detected"
assert isinstance(_best6.encoding, str), f"encoding is str: {type(_best6.encoding)!r}"
assert "utf" in _best6.encoding.lower(), f"encoding is utf: {_best6.encoding!r}"

# Rule 7: percent_chaos and percent_coherence are floats in [0, 1]
_result7 = charset_normalizer.from_bytes(b"simple english words here")
_best7 = _result7.best()
if _best7 is not None:
    assert isinstance(_best7.percent_chaos, float), \
        f"percent_chaos type = {type(_best7.percent_chaos)!r}"
    assert 0.0 <= _best7.percent_chaos <= 1.0, \
        f"percent_chaos range = {_best7.percent_chaos!r}"
    assert isinstance(_best7.percent_coherence, float), \
        f"percent_coherence type = {type(_best7.percent_coherence)!r}"

# Rule 8: Module attributes are identity-stable
_fb_ref = charset_normalizer.from_bytes
_det_ref = charset_normalizer.detect
_ib_ref = charset_normalizer.is_binary
for _ in range(5):
    assert charset_normalizer.from_bytes is _fb_ref, "from_bytes stable"
    assert charset_normalizer.detect is _det_ref, "detect stable"
    assert charset_normalizer.is_binary is _ib_ref, "is_binary stable"

print("behavior OK")
