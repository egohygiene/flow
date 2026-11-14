#!/usr/bin/env python3
"""Observe an existing release:prepare result and packaged-byte installed smoke.

This does not build, sign, publish or implement packaging. It reuses the
existing verifier and keeps all tamper cases in fresh test-owned copies.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import io
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
import tomllib

from check_released_read_only import (OWNER_IO, build_binary, canonical, command,
    document, envelope, require, sha)


def put(path, value):
    with path.open("xb") as writer: writer.write(canonical(value) + b"\n")


def git(source, *args):
    result = subprocess.run(["git", "-c", "core.hooksPath=/dev/null", *args],
        cwd=source, check=True, capture_output=True, timeout=30)
    require(len(result.stdout) <= 16_777_216, "bounded source observation required")
    return result.stdout


def exact_verifier_refusal(code, stdout, stderr, reason):
    final_line = stderr.splitlines(keepends=True)[-1:]
    require(type(code) is int and code == 1 and stdout == b""
            and final_line == [("ValueError: " + reason + "\n").encode()],
            "expected exact verifier exit and final refusal line differ")


def asset_inventory(entries):
    require(type(entries) is list and len(entries) == 8
            and all(type(entry) is dict and type(entry.get("name")) is str for entry in entries),
            "exact eight named package assets required")
    names = [entry["name"] for entry in entries]
    require(len(set(names)) == len(names), "duplicate package assets before projection")
    return {entry["name"]: entry for entry in entries}


def exact_lock_inventory(packages, locked):
    actual = [(p["name"], p["versionInfo"]) for p in packages]
    expected = [(p["name"], p["version"]) for p in locked]
    require(len(actual) == len(set(actual)) == len(expected) == len(set(expected))
            and set(actual) == set(expected), "SPDX inventory must match the unique owning lock packages")


def observe(args):
    root = args.output.absolute()
    require(root.parent.resolve() == root.parent and not root.exists(), "fresh private proof root required")
    root.mkdir(mode=0o700); cwd=root / "empty-cwd"; cwd.mkdir(mode=0o700)
    logs=root / "commands"; logs.mkdir(mode=0o700)
    owner=args.source / "scripts/release/package.py"
    require(owner.read_bytes() == git(args.source, "show", args.source_revision + ":scripts/release/package.py"),
            "executed verifier differs from the reviewed source owner")
    manifest_path=args.package / "flow-release-manifest-v1.json"
    manifest_bytes=OWNER_IO.regular(manifest_path,16_777_216)
    require(sha(manifest_bytes)==args.manifest_sha256, "explicit prepared manifest pin differs")
    manifest=document(manifest_bytes)
    require(manifest["source_revision"]==args.source_revision and manifest["native_build"]["isolated_builds"]==2,
            "exact source/two-build preparation record required")
    assets=asset_inventory(manifest["assets"])
    primary=manifest["primary_binary"]; descriptor_name=manifest["compiled_descriptor"]
    descriptor=document(OWNER_IO.regular(args.package/descriptor_name,1_048_576))
    controls=[]

    def verify(case, package, digest, reason=None):
        result=subprocess.run([sys.executable,"-B",str(owner),"verify","--package-directory",str(package),
            "--manifest-sha256",digest],cwd=cwd,capture_output=True,timeout=30)
        require(len(result.stdout)+len(result.stderr)<=1_048_576,"bounded verifier capture required")
        (logs/(case+".stdout")).write_bytes(result.stdout);(logs/(case+".stderr")).write_bytes(result.stderr)
        if reason is None:
            require(result.returncode==0 and not result.stderr and document(result.stdout)=={
                "publication_performed":False,"qualified":False,"verification":"exact_candidate_package_bytes_only"},
                "exact owner verification did not pass")
        else:
            exact_verifier_refusal(result.returncode,result.stdout,result.stderr,reason)
        controls.append({"case":case,"accepted":reason is None,"expected_reason":reason,
            "exit_code":result.returncode,"stdout_sha256":sha(result.stdout),"stderr_sha256":sha(result.stderr)})

    verify("original-package",args.package,args.manifest_sha256)
    binaries=[];identities=[]
    for index in (1,2):
        path=args.work/("build-"+str(index))/manifest["target"]/"release/flow"
        data=build_binary(path);metadata=path.stat();identities.append((metadata.st_dev,metadata.st_ino))
        require(sha(data)==assets[primary]["sha256"] and len(data)==assets[primary]["byte_count"],
                "isolated compiler output differs from packaged bytes")
        selected=root/("captured-build-"+str(index))
        with selected.open("xb") as writer:writer.write(data)
        selected.chmod(0o700)
        code,stdout,stderr=command([str(selected),"release-contract","--format","json"],cwd=cwd)
        value=envelope(stdout,stderr,code,"release-contract",0,"complete")
        require(value["result"]==descriptor,"independent compiled descriptor differs")
        (logs/("build-"+str(index)+"-contract.stdout.json")).write_bytes(stdout)
        binaries.append({"build":index,"sha256":sha(data),"byte_count":len(data),
                         "descriptor_sha256":sha(OWNER_IO.regular(args.package/descriptor_name,1_048_576))})
    require(identities[0]!=identities[1],"two distinct isolated compiler artifacts required")
    inventory=document(OWNER_IO.regular(args.package/"library-source-members.json",16_777_216))
    require(inventory["source_revision"]==args.source_revision,"source inventory revision differs")
    archive=OWNER_IO.regular(args.package/manifest["library_source"],134_217_728)
    members={};total=0;prefix="flow-"+manifest["version"]+"/"
    with tarfile.open(fileobj=io.BytesIO(archive),mode="r:gz") as stream:
        for entry in stream:
            require(entry.isfile() and entry.name.startswith(prefix) and ".." not in Path(entry.name).parts,
                    "source archive contains unsupported input")
            name=entry.name[len(prefix):];require(name not in members,"duplicate source member")
            reader=stream.extractfile(entry);require(reader is not None,"source member unreadable")
            data=reader.read(16_777_217);total+=len(data)
            require(len(data)<=16_777_216 and total<=134_217_728 and len(members)<8192,"source bounds exceeded")
            require(data==git(args.source,"show",args.source_revision+":"+name),"archived source differs from Git")
            members[name]=sha(data)
    require(members==inventory["members"],"source inventory/archive membership differs")
    lock=tomllib.loads(git(args.source,"show",args.source_revision+":Cargo.lock").decode())
    sbom=document(OWNER_IO.regular(args.package/"flow-sbom.spdx.json",16_777_216))
    require(sbom["spdxVersion"]=="SPDX-2.3","SPDX version differs")
    exact_lock_inventory(sbom["packages"],lock["package"])
    notices=OWNER_IO.regular(args.package/"THIRD_PARTY_NOTICES.txt",16_777_216).decode()
    require(all(p["name"]+" "+p["versionInfo"]+": "+p["licenseDeclared"]+"\n" in notices
                for p in sbom["packages"]),"declared notices omit an inventoried package")
    verify("wrong-manifest-pin",args.package,"0"*64,"exact manifest pin differs")
    changed=root/"changed-executable-package";shutil.copytree(args.package,changed)
    data=bytearray((changed/primary).read_bytes());data[-1]^=1;(changed/primary).write_bytes(data)
    verify("changed-executable",changed,args.manifest_sha256,"package asset changed")
    extra=root/"unknown-member-package";shutil.copytree(args.package,extra)
    (extra/"operator-note.txt").write_bytes(b"synthetic unknown member; retained\n")
    verify("unknown-package-member",extra,args.manifest_sha256,"package has missing or unreferenced members")
    promoted=root/"promoted-scope-package";shutil.copytree(args.package,promoted)
    value=dict(manifest);value["qualification"]="qualified"
    changed_manifest=canonical(value)+b"\n";(promoted/manifest_path.name).write_bytes(changed_manifest)
    verify("scope-promotion",promoted,sha(changed_manifest),"package cannot assert publication/qualification")
    installed=args.installation/"bin/flow";data=OWNER_IO.regular(installed,83_886_080)
    require(sha(data)==assets[primary]["sha256"],"installed smoke is not using packaged bytes")
    smoke=[]
    for operation in ("version","help","release-contract"):
        code,stdout,stderr=command([str(installed),operation,"--format","json"],cwd=cwd)
        value=envelope(stdout,stderr,code,operation,0,"complete")
        if operation=="release-contract":require(value["result"]==descriptor,"installed compiled descriptor differs")
        (logs/("installed-"+operation+".stdout.json")).write_bytes(stdout)
        smoke.append({"command":operation,"exit_code":code,"stdout_sha256":sha(stdout)})
    verify("original-package-preserved",args.package,args.manifest_sha256)
    require(list(cwd.iterdir())==[],"outside-checkout cwd changed")
    report={"schema_version":"flow.local-package-candidate-proof/v1","status":"passed",
        "observed_at":datetime.now(timezone.utc).isoformat(),"source_revision":args.source_revision,
        "target":manifest["target"],"host":{"system":platform.system(),"machine":platform.machine(),"libc":platform.libc_ver()},
        "rustc":manifest["native_build"]["rustc"],"manifest_sha256":args.manifest_sha256,
        "binaries":binaries,"independent_target_artifacts":True,"assets":manifest["assets"],
        "source_members_verified_against_git":len(members),"source_uncompressed_bytes":total,
        "spdx_lock_packages":len(sbom["packages"]),"controls":controls,"installed_smoke":smoke,
        "original_package_preserved":True,"command_cwd_outside_checkout":True,
        "release_published":False,"publisher_authenticated":False,"native_matrix_qualified":False,
        "registry_channel":"unavailable","hosted_ci_dispatched":False,
        "limits":"same-host isolated executable/descriptor reproducibility; package timestamp/SBOM metadata is not a cross-run bit-reproducible or authenticated release claim"}
    put(root/"report.json",report);return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ("source","package","work","installation","output"):
        parser.add_argument("--"+name,required=True,type=Path)
    parser.add_argument("--source-revision",required=True);parser.add_argument("--manifest-sha256",required=True)
    print(json.dumps(observe(parser.parse_args()),sort_keys=True))


if __name__=="__main__":main()
