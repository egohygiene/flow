"""Bound capture while supervising a test command, without shell strings or retries."""
import os
from pathlib import Path
import selectors
import signal
import subprocess
import sys
import time


class BudgetExceeded(ValueError):
    def __init__(self, message, output):
        super().__init__(message)
        self.output = output


def run(command, *, cwd, timeout=600, max_output=4_194_304):
    """Return merged output; on timeout/flood kill this test process group and reap."""
    output = bytearray()
    deadline = time.monotonic() + timeout
    with subprocess.Popen(command, cwd=cwd, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, start_new_session=True) as process:
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ)
                while selector.get_map():
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise BudgetExceeded("test command exceeded runtime budget", bytes(output))
                    for key, _ in selector.select(remaining):
                        data = os.read(key.fileobj.fileno(), 65_536)
                        if not data:
                            selector.unregister(key.fileobj)
                            break
                        available = max_output - len(output)
                        output.extend(data[:available])
                        if len(data) > available:
                            raise BudgetExceeded("test command exceeded output budget", bytes(output))
            try:
                code = process.wait(timeout=max(0, deadline - time.monotonic()))
            except subprocess.TimeoutExpired as error:
                raise BudgetExceeded("test command exceeded runtime budget", bytes(output)) from error
            if code:
                _kill_group(process)
            return subprocess.CompletedProcess(command, code, bytes(output).decode("utf-8", errors="replace"))
        except BaseException:
            _kill_group(process)
            process.wait()
            raise


def _kill_group(process):
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def cargo_command(cargo, root):
    """Limit test executables and their children, not the compiler/linker."""
    if sys.platform != "linux":
        raise ValueError("bounded scenario qualification requires Linux")
    version = subprocess.check_output([cargo, "--version", "--verbose"], text=True, timeout=10)
    host = next(line.split(": ", 1)[1] for line in version.splitlines() if line.startswith("host: "))
    import json
    runner = [sys.executable, str(Path(root) / "tools/bounded_test_runner.py")]
    return [cargo, "--config", f"target.{host}.runner={json.dumps(runner)}", "test", "--locked", "--offline"]
