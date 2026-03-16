Implemented the fix in the Astropy checkout at `/home/daytona/workspace/astropy-target`.

Changes:
- Made QDP `READ SERR` / `READ TERR` command parsing case-insensitive in `astropy/io/ascii/qdp.py`
- Added a regression test for lowercase commands in `astropy/io/ascii/tests/test_qdp.py`

Validation:
- Ran: `python -m pytest astropy/io/ascii/tests/test_qdp.py -q`
- Result: `9 passed`

Patch summary:
```diff
-    _command_re = r"READ [TS]ERR(\s+[0-9]+)+"
+    _command_re = r"(?i:READ [TS]ERR(\s+[0-9]+)+)"
```