# Official arXiv Baseline

Verified: 2026-08-02

Official sources:

- https://info.arxiv.org/help/submit_tex.html
- https://info.arxiv.org/help/ancillary_files.html

Verified baseline:

- current TeX processing uses Submission System 1.5
- TeX Live 2025 is the default, with TeX Live 2023 also listed at the
  verification date
- processor selection is supported
- PDF-mode figures include PDF, PNG, and JPG
- on-the-fly figure conversion is not performed
- embedded JavaScript is prohibited
- `.bib` processing and pre-generated `.bbl` files are supported with
  compatibility requirements
- hidden files and directories are deleted upon announcement
- extraneous source files should not be included
- ancillary content has a separate supported mechanism

These requirements are mutable. Re-check official documentation and
update the target record before release.
