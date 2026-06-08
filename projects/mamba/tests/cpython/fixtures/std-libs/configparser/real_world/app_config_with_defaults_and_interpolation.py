# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "real_world"
# case = "app_config_with_defaults_and_interpolation"
# subject = "configparser.ConfigParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ConfigParser: an end-to-end app-config flow: write a layered ini (DEFAULT + service sections using %(...)s interpolation) to a tempfile, read it back with read(), and resolve typed settings (host/port via getint, debug via getboolean, an interpolated log path) the way a service boot would"""
import configparser
import os
import tempfile

ini = (
    "[DEFAULT]\n"
    "root = /srv/app\n"
    "debug = no\n"
    "\n"
    "[server]\n"
    "host = 0.0.0.0\n"
    "port = 8080\n"
    "log = %(root)s/server.log\n"
    "\n"
    "[worker]\n"
    "debug = yes\n"
    "log = %(root)s/worker.log\n"
)

with tempfile.TemporaryDirectory() as tmp:
    path = os.path.join(tmp, "app.ini")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(ini)

    # Service boot: load the config file from disk.
    cfg = configparser.ConfigParser()
    read_files = cfg.read(path)
    assert read_files == [path], f"read returned = {read_files!r}"

    # Typed reads the way a boot path resolves settings.
    assert cfg.get("server", "host") == "0.0.0.0", "server host"
    assert cfg.getint("server", "port") == 8080, "server port (typed int)"
    assert cfg.getboolean("server", "debug") is False, "server inherits DEFAULT debug=no"
    assert cfg.getboolean("worker", "debug") is True, "worker overrides debug=yes"

    # Interpolation resolves the shared root into each per-service log path.
    assert cfg.get("server", "log") == "/srv/app/server.log", "server log interpolated"
    assert cfg.get("worker", "log") == "/srv/app/worker.log", "worker log interpolated"

print("app_config_with_defaults_and_interpolation OK")
