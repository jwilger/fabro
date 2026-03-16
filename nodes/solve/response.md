The fix is exactly one line change on line 523:

**Before:** `elif operand is None:`
**After:** `elif operand is None or operand.mask is None:`

This is the minimal fix. The bug was introduced in the commit referenced in the issue (`feeb716b7412c477c694648ee1e93be2c4a73700`) where the condition `elif operand.mask is None:` was accidentally changed to `elif operand is None:`. 

The logic should be:
1. If neither operand has a mask (or `handle_mask` is None) → return `None`
2. If `self` has no mask but `operand` does → return a copy of `operand.mask`
3. If `operand` is None (scalar) or `operand` has no mask but `self` does → return a copy of `self.mask`
4. Both have masks → apply `handle_mask` function

The old code on line 523 only checked `operand is None` (i.e., the operand is a scalar like `1.0`), but missed the case where `operand` is an NDDataRef object with `mask=None`. This caused the code to fall through to the `else` branch and call `handle_mask(self.mask, None)`, which fails with `TypeError: unsupported operand type(s) for |: 'int' and 'NoneType'`.