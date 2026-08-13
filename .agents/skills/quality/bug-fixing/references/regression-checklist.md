# Regression Protection Checklist

Use this checklist before claiming the bug is fixed.

- [ ] The original failure is reproduced or represented by the strongest safe proxy.
- [ ] The root cause explains both the symptom and the chosen fix.
- [ ] The fix is the smallest change that restores the intended invariant.
- [ ] A regression test would fail before the fix and pass after it.
- [ ] Neighboring code paths sharing the same invariant were checked proportionally.
- [ ] Validation results and any remaining uncertainty are reported explicitly.
