# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_robotparser"
# dimension = "surface"
# case = "api_robot_file_parser_is_present"
# subject = "urllib.robotparser.RobotFileParser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.robotparser.RobotFileParser: api_robot_file_parser_is_present (surface)."""
import urllib.robotparser

assert hasattr(urllib.robotparser, "RobotFileParser")
print("api_robot_file_parser_is_present OK")
