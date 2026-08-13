# Deterministic Test Checklist

Use this checklist while reviewing a test change.

- [ ] The chosen layer is the lowest one that gives enough confidence.
- [ ] Time, randomness, locale, filesystem, environment, and network dependencies are controlled.
- [ ] Assertions check observable behavior rather than incidental implementation detail.
- [ ] Flakiness is not hidden behind sleeps, retries, or disabled coverage.
- [ ] Bug fixes include regression protection.
- [ ] Remaining risk is identified when the test seam is still incomplete.
