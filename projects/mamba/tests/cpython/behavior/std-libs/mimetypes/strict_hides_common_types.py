# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "strict_hides_common_types"
# subject = "mimetypes.MimeTypes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.MimeTypes: the non-standard image/jpg alias is hidden under strict (default) but visible under strict=False for both guess_extension and guess_all_extensions"""
import mimetypes

db = mimetypes.MimeTypes()

# image/jpg is a non-standard alias: only visible when strict=False.
assert db.guess_all_extensions("image/jpg", strict=True) == [], "jpg strict"
assert db.guess_all_extensions("image/jpg", strict=False) == [".jpg"], "jpg loose"
assert db.guess_extension("image/jpg", strict=True) is None, "jpg ext strict"
assert db.guess_extension("image/jpg", strict=False) == ".jpg", "jpg ext loose"
print("strict_hides_common_types OK")
