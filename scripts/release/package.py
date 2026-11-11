#!/usr/bin/env python3
"""Prepare reviewed Flow artifacts. No downloads, signing or publication.

Native build/package is explicit. plan/verify/handoff never invoke the product.
The source archive is Git-owned library source, not a crates.io publication.
"""
from __future__ import annotations
import argparse
import datetime
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import stat
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = "flow.release-manifest/v1"
MANIFEST = "flow-release-manifest-v1.json"
MAX_FILE = 536870912
MAX_JSON = 16777216
MAX_SOURCE = 134217728


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path, maximum=MAX_FILE, allow_build_links=False):
    path = Path(path).absolute()
    require(path.resolve() == path, "canonical regular input required")
    before = path.stat()
    require(stat.S_ISREG(before.st_mode) and (allow_build_links or before.st_nlink == 1) and 0 < before.st_size <= maximum, "bounded regular input required")
    with path.open("rb") as reader:
        data = reader.read(maximum + 1)
    after = path.stat()
    identity = lambda m: (m.st_dev, m.st_ino, m.st_size, m.st_mtime_ns, m.st_ctime_ns)
    require(identity(before) == identity(after) and len(data) == before.st_size, "input changed during observation")
    return data


def run(argv, source, env=None, timeout=120):
    result = subprocess.run([str(v) for v in argv], cwd=source, env=env, capture_output=True, check=False, timeout=timeout)
    require(result.returncode == 0 and len(result.stdout) + len(result.stderr) <= MAX_JSON, "release preparation command failed or exceeded capture bound")
    return result.stdout


def git(source, *args):
    return run(["git", "-c", "core.hooksPath=/dev/null", *args], source)


def authority(source):
    package = tomllib.loads(read(source / "Cargo.toml", MAX_JSON).decode())["package"]
    require(package["name"] == "flow" and package.get("publish") is False and package.get("default-run") == "flow", "Flow Cargo authority/channel/default differs")
    require(re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", package["version"]), "exact Cargo version required")
    config = json.loads(read(source / ".egohygiene/release-build.json", MAX_JSON))
    require(config["schema_version"] == "flow.release-build/v1" and config["primary_binary"] == "flow" and config["registry_channel"] == "unavailable", "Flow release build declaration differs")
    return package, config


def clean(source, revision):
    require(re.fullmatch(r"[0-9a-f]{40}", revision) and git(source, "rev-parse", "HEAD").decode().strip() == revision, "exact reviewed source HEAD required")
    require(not git(source, "status", "--porcelain", "--untracked-files=all"), "clean source required")


def host_target():
    system, machine = platform.system(), platform.machine().lower()
    if system == "Linux" and machine in {"x86_64", "amd64"} and platform.libc_ver()[0] == "glibc": return "x86_64-unknown-linux-gnu"
    if system == "Darwin" and machine in {"arm64", "aarch64"}: return "aarch64-apple-darwin"
    if system == "Darwin" and machine in {"x86_64", "amd64"}: return "x86_64-apple-darwin"
    raise ValueError("native host adapter unavailable; retained targets stay visible")


def fresh(value, source):
    value = Path(value).absolute()
    require(value.parent.resolve() == value.parent and value.parent.is_dir() and not value.is_relative_to(source), "fresh external output with canonical parent required")
    value.mkdir(mode=0o700)
    return value


def put(path, data, mode=0o644):
    with path.open("xb") as writer:
        writer.write(data)
        writer.flush()
        os.fsync(writer.fileno())
    path.chmod(mode)


def json_bytes(value):
    return (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()


def source_archive(source, revision, version):
    paths = git(source, "ls-tree", "-r", "-z", revision).split(b"\0")
    files, total = {}, 0
    for item in paths:
        if not item: continue
        metadata, path_bytes = item.split(b"\t", 1)
        mode, kind, blob = metadata.decode().split()
        name = path_bytes.decode()
        selected = name in {"Cargo.toml", "Cargo.lock", "LICENSE", "README.md", "CHANGELOG.md"} or name.startswith(("src/", "schemas/", "tests/", "docs/", "examples/"))
        if not selected: continue
        require(mode in {"100644", "100755"} and kind == "blob" and ".." not in Path(name).parts, "source archive contains unsupported linked input")
        data = git(source, "cat-file", "blob", blob)
        total += len(data)
        require(total <= MAX_SOURCE and len(files) < 8192, "source archive bound")
        files[name] = (data, 0o755 if mode == "100755" else 0o644)
    require({"src/lib.rs", "src/bin/flow.rs", "Cargo.toml", "Cargo.lock", "LICENSE"}.issubset(files), "library source archive missing required authority/source")
    buffer = io.BytesIO()
    with gzip.GzipFile(fileobj=buffer, mode="wb", filename="", mtime=0) as zipped:
        with tarfile.open(fileobj=zipped, mode="w", format=tarfile.PAX_FORMAT) as archive:
            for name, (data, mode) in sorted(files.items()):
                entry = tarfile.TarInfo("flow-" + version + "/" + name)
                entry.size, entry.mode, entry.mtime = len(data), mode, 0
                archive.addfile(entry, io.BytesIO(data))
    return buffer.getvalue(), {name: sha(data) for name, (data, _) in sorted(files.items())}


def sbom(source, version, revision, metadata, target):
    lock = tomllib.loads(read(source / "Cargo.lock", MAX_JSON).decode())
    details = {(p["name"],p["version"]):p for p in metadata["packages"]}
    packages = []
    for index, package in enumerate(sorted(lock["package"], key=lambda p:(p["name"],p["version"])),1):
        info = details.get((package["name"],package["version"]),{})
        record = {"SPDXID":"SPDXRef-Package-"+str(index),"name":package["name"],"versionInfo":package["version"],
            "downloadLocation":"NOASSERTION","filesAnalyzed":False,"licenseConcluded":"NOASSERTION", "licenseDeclared":info.get("license") or "NOASSERTION", "copyrightText":"NOASSERTION"}
        if package.get("checksum"): record["checksums"]=[{"algorithm":"SHA256","checksumValue":package["checksum"]}]
        packages.append(record)
    created=datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="microseconds").replace("+00:00","Z")
    return {"spdxVersion":"SPDX-2.3","dataLicense":"CC0-1.0","SPDXID":"SPDXRef-DOCUMENT","name":"flow-"+version,
        "documentNamespace":"https://github.com/egohygiene/flow/sbom/"+revision+"/"+target+"/"+sha(created.encode()),
        "creationInfo":{"created":created,"creators":["Tool: flow-source-release-preparation"],"comment":"Inventory of Cargo.lock and supplied frozen metadata. Includes dev/optional packages; not a native binary or OS/transitive tool composition claim."},
        "packages":packages,"relationships":[{"spdxElementId":"SPDXRef-DOCUMENT","relationshipType":"DESCRIBES","relatedSpdxElement":p["SPDXID"]} for p in packages]}


def prepare(args):
    source = Path(args.source_dir).resolve()
    package, config = authority(source)
    clean(source, args.source_revision)
    require(args.target in config["native_targets"] and args.target == host_target(), "native target/host differs")
    rustc = run(["rustup","run",config["rust_toolchain"],"rustc","--version","--verbose"],source).decode()
    require("host: "+args.target+"\n" in rustc and rustc.startswith("rustc 1.85."), "installed native toolchain differs")
    cohort_bytes = read(Path(args.cohort),1048576)
    cohort = json.loads(cohort_bytes)
    require(cohort["schema_version"] == "flow.release-cohort/v1" and cohort["flow_version"] == package["version"] and cohort["rows"], "supplied cohort/version differs")
    work = fresh(args.work_directory,source)
    output = Path(args.output).absolute()
    require(not output.is_relative_to(work) and not work.is_relative_to(output), "separate work/output roots required")
    binaries, descriptors = [], []
    for index in [1,2]:
        target = work / ("build-"+str(index))
        env = dict(os.environ)
        env.pop("RUSTFLAGS",None)
        env.update({"RUSTUP_TOOLCHAIN":config["rust_toolchain"],"RUSTUP_AUTO_INSTALL":"0","CARGO_TARGET_DIR":str(target),"CARGO_NET_OFFLINE":"true","CARGO_INCREMENTAL":"0",
            "CARGO_ENCODED_RUSTFLAGS":"--remap-path-prefix="+str(source)+"=/flow/source\x1f--remap-path-prefix="+str(target)+"=/flow/target"})
        run(["cargo","build","--frozen","--release","--bin","flow","--target",args.target],source,env,2400)
        executable = target / args.target / "release/flow"
        binary = read(executable,allow_build_links=True)
        envelope = json.loads(run([executable,"release-contract","--format","json"],source,env))
        require(envelope["schema_version"] == "flow.command-result/v1" and envelope["command"] == "release-contract" and envelope["outcome"] == "complete" and envelope["exit_code"] == 0, "compiled release envelope differs")
        descriptor = envelope["result"]
        require(descriptor["schema_version"] == "flow.release-contract/v1" and descriptor["version"] == package["version"] and descriptor["primary_binary"] == "flow" and descriptor["qualification"] == "not_established_by_contract_description", "compiled Flow identity differs")
        binaries.append(binary)
        descriptors.append(json_bytes(descriptor))
    require(binaries[0] == binaries[1] and descriptors[0] == descriptors[1], "isolated native binary/contract preparation differs")
    metadata = json.loads(run(["cargo","metadata","--frozen","--format-version","1","--filter-platform",args.target],source,env))
    library, members = source_archive(source,args.source_revision,package["version"])
    clean(source,args.source_revision)
    output = fresh(output,source)
    assets = {}
    def asset(name,data,kind,mode=0o644):
        put(output/name,data,mode)
        assets[name]={"name":name,"sha256":sha(data),"byte_count":len(data),"kind":kind}
    binary_name = "flow-"+args.target
    asset(binary_name,binaries[0],"cli-binary",0o755)
    asset("flow-release-contract-v1.json",descriptors[0],"compiled-contract")
    asset("flow-"+package["version"]+"-source.tar.gz",library,"library-source")
    asset("library-source-members.json",json_bytes({"source_revision":args.source_revision,"members":members}),"library-source-inventory")
    asset("provider-cohort.json",cohort_bytes,"provider-cohort")
    dependency_inventory=sbom(source,package["version"],args.source_revision,metadata,args.target)
    asset("flow-sbom.spdx.json",json_bytes(dependency_inventory),"spdx-sbom")
    notices="Flow dependency/license declarations; not license adjudication or a complete native/OS/model inventory.\n"+"\n".join(p["name"]+" "+p["versionInfo"]+": "+p["licenseDeclared"] for p in dependency_inventory["packages"])+"\n"
    asset("THIRD_PARTY_NOTICES.txt",notices.encode(),"dependency-notices")
    asset("LICENSE",read(source/"LICENSE",MAX_JSON),"product-license")
    manifest={"schema_version":SCHEMA,"repository":"egohygiene/flow","version":package["version"],"source_revision":args.source_revision,"target":args.target,
        "primary_binary":binary_name,"compiled_descriptor":"flow-release-contract-v1.json","provider_cohort":"provider-cohort.json","library_source":"flow-"+package["version"]+"-source.tar.gz",
        "assets":[assets[k] for k in sorted(assets)],"native_build":{"host_target":host_target(),"rustc":rustc,"isolated_builds":2},
        "retained_target_objectives":config["native_targets"]+config["retained_target_objectives"],"registry_channel":"unavailable","qualification":"not_established_by_preparation","publication":"not_performed"}
    put(output/MANIFEST,json_bytes(manifest))
    inventory={**assets,MANIFEST:{"sha256":sha(read(output/MANIFEST,MAX_JSON))}}
    put(output/"SHA256SUMS","".join(inventory[n]["sha256"]+"  "+n+"\n" for n in sorted(inventory)).encode())


def verify(directory, expected):
    directory=Path(directory).absolute()
    require(directory.resolve()==directory and directory.is_dir(),"canonical package directory required")
    data=read(directory/MANIFEST,MAX_JSON)
    require(sha(data)==expected,"exact manifest pin differs")
    manifest=json.loads(data)
    require(manifest["schema_version"]==SCHEMA and manifest["repository"]=="egohygiene/flow" and manifest["qualification"]=="not_established_by_preparation" and manifest["publication"]=="not_performed", "package cannot assert publication/qualification")
    require(re.fullmatch(r"[0-9a-f]{40}",manifest["source_revision"]) and manifest["target"] in {"x86_64-unknown-linux-gnu","aarch64-apple-darwin","x86_64-apple-darwin"} and manifest["registry_channel"]=="unavailable", "package source/target/channel differs")
    require(0 < len(manifest["assets"]) <= 64 and len({a["name"] for a in manifest["assets"]})==len(manifest["assets"]),"package inventory bounds/identity")
    inventory={MANIFEST:{"sha256":expected}}
    for item in manifest["assets"]:
        require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*",item["name"]),"unsafe package asset name")
        content=read(directory/item["name"])
        require(sha(content)==item["sha256"] and len(content)==item["byte_count"],"package asset changed")
        inventory[item["name"]]=item
    require({p.name for p in directory.iterdir()}==set(inventory)|{"SHA256SUMS"},"package has missing or unreferenced members")
    require(read(directory/"SHA256SUMS",MAX_JSON)=="".join(inventory[n]["sha256"]+"  "+n+"\n" for n in sorted(inventory)).encode(),"package checksum inventory differs")
    require(manifest["primary_binary"] in inventory and manifest["compiled_descriptor"] in inventory and manifest["provider_cohort"] in inventory and manifest["library_source"] in inventory,"package authority assets missing")
    require(manifest["primary_binary"]=="flow-"+manifest["target"] and manifest["compiled_descriptor"]=="flow-release-contract-v1.json" and manifest["provider_cohort"]=="provider-cohort.json" and manifest["library_source"]=="flow-"+manifest["version"]+"-source.tar.gz", "package authority asset roles differ")
    require(inventory[manifest["primary_binary"]]["kind"]=="cli-binary" and inventory[manifest["library_source"]]["kind"]=="library-source", "package executable/library kinds differ")
    descriptor=json.loads(read(directory/manifest["compiled_descriptor"],MAX_JSON))
    require(descriptor["schema_version"]=="flow.release-contract/v1" and descriptor["version"]==manifest["version"] and descriptor["primary_binary"]=="flow", "compiled descriptor does not bind package")
    cohort=json.loads(read(directory/manifest["provider_cohort"],1048576))
    require(cohort["schema_version"]=="flow.release-cohort/v1" and cohort["flow_version"]==manifest["version"],"package provider cohort differs")
    return manifest


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action",choices=["plan","prepare","verify","handoff"])
    parser.add_argument("--source-dir",default=str(ROOT))
    parser.add_argument("--source-revision")
    parser.add_argument("--target")
    parser.add_argument("--cohort")
    parser.add_argument("--work-directory")
    parser.add_argument("--output")
    parser.add_argument("--package-directory")
    parser.add_argument("--manifest-sha256")
    args=parser.parse_args()
    if args.action=="plan":
        package,config=authority(Path(args.source_dir).resolve())
        print(json.dumps({"version":package["version"],"build":config,"published":False,"qualified":False},sort_keys=True))
    elif args.action=="prepare":
        require(all([args.source_revision,args.target,args.cohort,args.work_directory,args.output]),"prepare requires explicit revision/target/cohort/fresh work/output")
        prepare(args)
    else:
        require(args.package_directory and args.manifest_sha256,"package directory and exact manifest digest required")
        manifest=verify(args.package_directory,args.manifest_sha256)
        result={"verification":"exact_candidate_package_bytes_only","publication_performed":False,"qualified":False}
        if args.action=="handoff":
            result["required_gates"]="reviewed version/tag, existing native/install/supply-chain qualification and explicit maintainer publication; no gate is satisfied by this handoff"
            result["manual_argv"]=["gh","release","create","v"+manifest["version"],"--repo","egohygiene/flow","--target",manifest["source_revision"],"--draft","--latest=false"]+[str(p) for p in sorted(Path(args.package_directory).iterdir())]
        print(json.dumps(result,sort_keys=True))


if __name__=="__main__":
    main()
