#!/usr/bin/env python3
"""Compare exact tracked inputs for reuse of the measured Flow package.

This local Git observation does not authenticate source, rebuild, reseal the
package or promote a candidate into a native/released qualification.
"""
import argparse
from pathlib import Path
import posixpath
import re
import subprocess

from check_native_publication import put
from check_released_read_only import canonical, require, sha
from installed_audio_candidate_subjects import CONTEXT_REVISION, CONTEXT_TREE, CONTEXT_SCOPE

BUILD_REVISION = "69eafaf74af8891bde1c7a769fb57dbca815476c"
BUILD_TREE = "376f175971da85fa7e894efc06a3ac836ce8b519"


def permitted(path):
    return path.startswith(("docs/", "tools/")) or path in ["Taskfile.yml", "config/local-ci.json"]


def observe(repository):
    def git(*args):
        p = subprocess.run(["git", "-c", "core.hooksPath=/dev/null", *args], cwd=repository,
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, check=False)
        require(p.returncode == 0 and len(p.stdout) < 16777216, "bounded exact Git input observation failed")
        return p.stdout
    def inventory(revision):
        result = {}
        for item in git("ls-tree", "-r", "-z", revision).split(b"\0"):
            if not item: continue
            meta, raw = item.split(b"\t", 1); mode, kind, digest = meta.decode().split()
            path = raw.decode("utf-8")
            require(kind == "blob" and len(result) < 4096, "bounded regular tracked source objects required")
            result[path] = {"mode": mode, "type": kind, "sha": digest}
        return result
    require(git("rev-parse", BUILD_REVISION + "^{tree}").decode().strip() == BUILD_TREE
        and git("rev-parse", CONTEXT_REVISION + "^{tree}").decode().strip() == CONTEXT_TREE,
        "actual measured/selected Git trees differ")
    before, after = inventory(BUILD_REVISION), inventory(CONTEXT_REVISION)
    changed = sorted(p for p in set(before) | set(after) if before.get(p) != after.get(p))
    product_before = {p: v for p, v in before.items() if not permitted(p)}
    product_after = {p: v for p, v in after.items() if not permitted(p)}
    require(product_before == product_after and all(permitted(p) for p in changed),
        "selected source changes compiled input outside exact nonruntime exceptions")
    includes = []
    for path in sorted(p for p in after if p.endswith(".rs") and not permitted(p)):
        source = git("show", CONTEXT_REVISION + ":" + path).decode()
        macros = list(re.finditer(r"\b(include(?:_bytes|_str)!)\s*\(\s*\"([^\"]+)\"\s*\)", source))
        require(len(macros) == len(re.findall(r"\binclude(?:_bytes|_str)!\s*\(", source)),
            "dynamic include requires independent source-input analysis")
        for found in macros:
            target = posixpath.normpath(posixpath.join(posixpath.dirname(path), found.group(2)))
            require(target in before and target in after and before[target] == after[target]
                and target not in changed, "changed included bytes cannot reuse the old executable")
            includes.append({"owner": path, "kind": found.group(1), "target": target,
                "object": after[target]})
    # External config/task references in compiled source or Cargo/build inputs
    # need an explicit review instead of broadening the exclusion silently.
    for path in sorted(product_after):
        if not (path.startswith(("src/", ".cargo/")) or path in ["Cargo.toml", "build.rs"]): continue
        raw = git("show", CONTEXT_REVISION + ":" + path)
        require(b"config/local-ci.json" not in raw and b"Taskfile.yml" not in raw,
            "selected excluded control file is referenced by product inputs")
    return {"schema_version": "flow.audio-candidate-build-context/v1",
        "actual_build_revision": BUILD_REVISION, "actual_build_tree": BUILD_TREE,
        "selected_revision": CONTEXT_REVISION, "selected_tree": CONTEXT_TREE,
        "permitted_nonruntime_prefixes": ["docs/", "tools/"],
        "permitted_nonruntime_paths": ["Taskfile.yml", "config/local-ci.json"],
        "changed_paths": changed, "changed_compilation_inputs": [], "runtime_inputs_equal": True,
        "changed_nonruntime_files_embedded": False, "compilation_input_count": len(product_after),
        "compilation_input_inventory": product_after,
        "compilation_input_inventory_sha256": sha(canonical(product_after)),
        "include_reference_count": len(includes), "include_reference_inventory_sha256": sha(canonical(includes)),
        "include_references": includes, "source_authentication_performed": False,
        "scope": CONTEXT_SCOPE}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    value = observe(args.repository)
    put(args.output, value)
    print(canonical(value).decode())
