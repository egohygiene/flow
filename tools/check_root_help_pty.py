"""Observe explicitly selected local CLIs through real POSIX pseudo-terminals.

The driver never builds, installs, probes media, or approves wordmark design.
Inputs name canonical immutable binaries/assets and exact SHA-256 pins. Native
captures remain private; the portable report contains hashes and fixed labels.
"""
from __future__ import annotations

import argparse
import errno
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import selectors
import signal
import struct
import subprocess
import tempfile
import termios
import time

TOOLS = ("flow", "aniflow", "optiflow", "renderflow")
MACHINE = {
    "flow": ["release-contract", "--format", "json"],
    "aniflow": ["--output", "json", "release-contract"],
    "optiflow": ["--json", "release-contract"],
    "renderflow": ["release-contract"],
}
SCHEMAS = {"flow": "flow.command-result/v1", "aniflow": 1,
           "optiflow": "optiflow.command-result.v1", "renderflow": "renderflow.release-contract/v2"}
LIMIT = 1_048_576


def require(condition, message):
    if not condition:
        raise ValueError(message)


def identity(path, pin):
    require(path.is_absolute() and path.resolve(strict=True) == path and path.is_file()
            and not path.is_symlink(), "canonical regular input required")
    with path.open("rb") as file:
        digest = hashlib.file_digest(file, "sha256").hexdigest()
    require(isinstance(pin, str) and len(pin) == 64 and digest == pin, "selected input pin differs")
    return {"sha256": digest, "byte_count": path.stat().st_size}


def capture(binary, argv, cwd, interactive, changes=None):
    environment = {"PATH": "/usr/bin:/bin", "LC_ALL": "C.UTF-8", "TERM": "xterm-256color", "COLUMNS": "100"}
    for key, value in (changes or {}).items():
        if value is None:
            environment.pop(key, None)
        else:
            environment[key] = value
    master = slave = None
    if interactive:
        master, slave = pty.openpty()
        attributes = termios.tcgetattr(slave)
        attributes[1] &= ~termios.OPOST  # Compare literal native LF bytes, not terminal CRLF translation.
        termios.tcsetattr(slave, termios.TCSANOW, attributes)
        columns = environment.get("COLUMNS", "")
        terminal_width = int(columns) if columns.isdigit() and 1 <= int(columns) <= 1000 else 100
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, terminal_width, 0, 0))
    child = None
    buffers = {"stdout": bytearray(), "stderr": bytearray()}
    try:
        child = subprocess.Popen([str(binary), *argv], stdin=subprocess.DEVNULL,
                                 stdout=slave if interactive else subprocess.PIPE,
                                 stderr=subprocess.PIPE, cwd=cwd, env=environment, start_new_session=True)
        if slave is not None:
            os.close(slave); slave = None
        with selectors.DefaultSelector() as selector:
            selector.register(master if interactive else child.stdout, selectors.EVENT_READ, "stdout")
            selector.register(child.stderr, selectors.EVENT_READ, "stderr")
            deadline = time.monotonic() + 15
            while selector.get_map():
                require(time.monotonic() < deadline, "selected CLI deadline exceeded")
                for key, _ in selector.select(min(0.1, max(0, deadline - time.monotonic()))):
                    descriptor = key.fileobj if isinstance(key.fileobj, int) else key.fileobj.fileno()
                    try:
                        chunk = os.read(descriptor, 65_536)
                    except OSError as error:
                        if interactive and key.data == "stdout" and error.errno == errno.EIO:
                            chunk = b""
                        else:
                            raise
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    buffers[key.data].extend(chunk)
                    require(len(buffers[key.data]) <= LIMIT, "selected CLI output limit exceeded")
            code = child.wait(timeout=max(0.001, deadline - time.monotonic()))
        return code, bytes(buffers["stdout"]), bytes(buffers["stderr"])
    finally:
        if child is not None and child.poll() is None:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait()
        for descriptor in (slave, master):
            if descriptor is not None:
                os.close(descriptor)
        if child is not None:
            for stream in (child.stdout, child.stderr):
                if stream is not None:
                    stream.close()


def snapshot(value):
    code, stdout, stderr = value
    return {"exit_code": code, "stdout": {"sha256": hashlib.sha256(stdout).hexdigest(), "byte_count": len(stdout)},
            "stderr": {"sha256": hashlib.sha256(stderr).hexdigest(), "byte_count": len(stderr)}}


def unstyled(raw):
    # Accept only ordinary SGR styling on explicitly color-capable paths.
    plain = re.sub(rb"\x1b\[[0-9;]*m", b"", raw)
    require(b"\x1b" not in plain, "unexpected terminal control sequence")
    return plain


def compare(observed, baseline, mark, decorated, allow_color=False):
    require(observed[0] == baseline[0], "terminal changed native exit semantics")
    if allow_color:
        observed = observed[0], unstyled(observed[1]), unstyled(observed[2])
    else:
        require(b"\x1b" not in observed[1] + observed[2], "unexpected terminal escape sequence")
    require(observed[2] == baseline[2], "terminal changed native stderr semantics")
    if decorated:
        require(observed[1].count(mark) == 1, "exact static wordmark missing or repeated")
        require(observed[1].replace(mark, b"", 1) == baseline[1], "decoration changed semantic help")
    else:
        require(mark not in observed[1] + observed[2], "wordmark escaped interactive root-help boundary")
        require(observed == baseline, "plain fallback changed native bytes")


def check_tool(tool, selected, cwd):
    require(set(selected) == {"binary", "binary_sha256", "wordmark", "wordmark_sha256"}, "closed tool selection required")
    binary, asset = Path(selected["binary"]), Path(selected["wordmark"])
    binary_identity = identity(binary, selected["binary_sha256"])
    asset_identity = identity(asset, selected["wordmark_sha256"])
    require(asset_identity["byte_count"] <= 4096, "wordmark exceeds bound")
    mark = asset.read_bytes()
    require(mark and mark.isascii() and b"\x1b" not in mark
            and all(len(line) <= 72 for line in mark.splitlines()), "static ASCII wordmark policy differs")
    cases = []
    for argv, label in ((["--help"], "long"), (["-h"], "short")):
        plain = capture(binary, argv, cwd, False)
        require(plain[0] == 0 and not plain[2] and tool.encode() in plain[1]
                and b"Commands:" in plain[1], "semantic root help missing")
        require(mark not in plain[1] and b"\x1b" not in plain[1], "redirected root help decorated")
        cases.append({"id": label + "-redirected", **snapshot(plain)})
        for name, changes, decorated in (
            ("wide", {}, True), ("narrow", {"COLUMNS": "40"}, False),
            ("dumb", {"TERM": "dumb"}, False), ("unknown-term", {"TERM": "unrecognized"}, False),
            ("missing-term", {"TERM": None}, False), ("missing-width", {"COLUMNS": None}, False),
            ("invalid-width", {"COLUMNS": "100;unsafe"}, False),
            ("no-color", {"NO_COLOR": ""}, False), ("clicolor-zero", {"CLICOLOR": "0"}, False),
        ):
            observed = capture(binary, argv, cwd, True, changes)
            compare(observed, plain, mark, decorated, allow_color=decorated)
            cases.append({"id": label + "-" + name, **snapshot(observed)})
    subhelp = ["doctor", "--help"] if tool == "flow" else ["diagnostics", "--help"]
    for name, argv, code in (
        ("semantic-help-command", ["help"], 0), ("version", ["--version"], 0),
        ("parse-error", ["--unknown"], 2), ("subcommand-help", subhelp, 2 if tool == "flow" else 0),
        ("unsupported-completion-help", ["help", "completion"], 2),
        ("unsupported-man-help", ["help", "man"], 2),
        ("machine-contract", MACHINE[tool], 0),
    ):
        baseline = capture(binary, argv, cwd, False)
        require(baseline[0] == code and bool(baseline[1] or baseline[2]), "unexpected native command behavior")
        observed = capture(binary, argv, cwd, True)
        compare(observed, baseline, mark, False, allow_color=name != "machine-contract")
        if name == "machine-contract":
            value = json.loads(observed[1])
            require(not observed[2] and value.get("schema_version", value.get("schema")) == SCHEMAS[tool],
                    "native machine contract differs")
        cases.append({"id": name, **snapshot(observed)})
    require(identity(binary, selected["binary_sha256"]) == binary_identity
            and identity(asset, selected["wordmark_sha256"]) == asset_identity, "selected input changed")
    require(not list(cwd.iterdir()), "presentation operation wrote an unexpected file")
    return {"tool": tool, "state": "passed", "binary": binary_identity, "wordmark": asset_identity,
            "cases": cases, "executed_calls": 2 * 10 + 2 * 7}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    require(not args.output.exists() and not args.output.is_symlink() and args.output.parent.is_dir(), "fresh output required")
    selections = json.loads(args.inputs.read_bytes())
    require(set(selections) == set(TOOLS), "select all four independent tools exactly once")
    rows = []
    with tempfile.TemporaryDirectory(prefix="flow-root-help-") as directory:
        root = Path(directory)
        for tool in TOOLS:
            cwd = root / tool; cwd.mkdir()
            try:
                rows.append(check_tool(tool, selections[tool], cwd))
            except (ValueError, OSError, TypeError, KeyError, subprocess.TimeoutExpired):
                # Portable failures never interpolate native captures, paths or caller values.
                rows.append({"tool": tool, "state": "failed", "reason": "selected terminal conformance failed"})
    passed = all(row["state"] == "passed" for row in rows)
    report = {"schema_version": "flow.root-help-pty-observation/v1", "tools": rows, "all_selected_passed": passed,
              "platform_scope": "local POSIX PTY with supplied TERM and COLUMNS",
              "design_approved": False, "screen_reader_qualified": False, "release_qualified": False,
              "generators": "completion and man generators absent; unsupported help requests observed"}
    with args.output.open("x", encoding="utf8") as file:
        json.dump(report, file, indent=2); file.write("\n")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
