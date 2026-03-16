Implemented support for `header_rows` in `ascii.rst`.

What changed:
- `astropy/io/ascii/rst.py`
  - `RST.__init__` now accepts `header_rows`
  - adjusts `data.start_line` based on the number of header rows
  - uses the generated separator line at the correct index when wrapping the table with top/bottom RST borders
- `astropy/io/ascii/tests/test_rst.py`
  - added a regression test for writing an RST table with `header_rows=["name", "unit"]`

Result:
```python
tbl.write(sys.stdout, format="ascii.rst", header_rows=["name", "unit"])
```
now works and outputs:
```rst
===== ========
 wave response
   nm       ct
===== ========
350.0      0.7
950.0      1.2
===== ========
```

Validation:
- Ran `python -m pytest astropy/io/ascii/tests/test_rst.py -q`
- Result: `10 passed`