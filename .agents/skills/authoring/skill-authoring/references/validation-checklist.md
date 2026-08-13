# Skill Authoring Validation Checklist

Use this checklist before finalizing a canonical skill package.

- [ ] `name` matches the directory exactly and is stable kebab-case.
- [ ] `description` states both what the skill does and when it should load.
- [ ] `license` and Aether metadata are explicit.
- [ ] References, templates, and scripts are necessary rather than decorative.
- [ ] Links resolve locally and do not escape the repository root.
- [ ] Provenance and source-delta decisions are documented.
- [ ] Evals cover positive, negative, insufficient-evidence, boundary, and update behavior.
- [ ] No provider-specific instructions are embedded in the core workflow.
