#!/usr/bin/env python3
"""Cargo target runner for Linux conformance tests; inherited per-process limits."""
import os
import resource
import sys

ADDRESS_SPACE_BYTES = 4_294_967_296
FILE_BYTES = 16_777_216


def limit(kind, ceiling):
    _, hard = resource.getrlimit(kind)
    value = ceiling if hard == resource.RLIM_INFINITY else min(ceiling, hard)
    resource.setrlimit(kind, (value, value))


def main():
    limit(resource.RLIMIT_AS, ADDRESS_SPACE_BYTES)
    limit(resource.RLIMIT_FSIZE, FILE_BYTES)
    limit(resource.RLIMIT_CORE, 0)
    os.execv(sys.argv[1], sys.argv[1:])


if __name__ == "__main__":
    main()
