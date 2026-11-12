#!/usr/bin/env python3
"""Explicit pinned acquisition or single-member extraction; never execute assets."""
import argparse
import gzip
import hashlib
import os
from pathlib import Path
import re
import tarfile
import urllib.parse
import urllib.request

MAX_FILE=536870912
MAX_EXPANDED=1073741824
def require(value,message):
    if not value: raise ValueError(message)
def digest(path):
    result=hashlib.sha256();total=0
    with path.open("rb") as source:
        while data:=source.read(65536):
            total+=len(data);require(total<=MAX_FILE,"source asset bound");result.update(data)
    return result.hexdigest()
def output(path):
    path=Path(path).absolute()
    require(path.parent.resolve()==path.parent and path.parent.is_dir(),"canonical existing output parent required")
    return os.fdopen(os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),"wb")
def acquire(args):
    require(args.allow_network,"acquisition requires explicit --allow-network")
    url=urllib.parse.urlsplit(args.url)
    require(url.scheme=="https" and url.netloc=="github.com" and not url.query and not url.fragment and re.fullmatch(r"/egohygiene/(flow|aniflow|optiflow|renderflow)/releases/download/v[0-9A-Za-z.+-]+/[A-Za-z0-9][A-Za-z0-9_.-]*",url.path),"exact supported HTTPS release asset required")
    result=hashlib.sha256();total=0
    with urllib.request.urlopen(urllib.request.Request(args.url,headers={"User-Agent":"flow-explicit-asset-preparation"}),timeout=60) as response:
        final=urllib.parse.urlsplit(response.geturl())
        require(final.scheme=="https" and final.hostname in {"github.com","objects.githubusercontent.com","release-assets.githubusercontent.com"},"release redirect host/scheme refused")
        with output(args.output) as target:
            while data:=response.read(65536):
                total+=len(data);require(total<=MAX_FILE,"download bound exceeded; preserve partial output");target.write(data);result.update(data)
            require(total>0 and result.hexdigest()==args.sha256,"download bytes differ; preserve output for inspection")
            target.flush();os.fsync(target.fileno())
class Expanded:
    def __init__(self,stream): self.stream,self.total=stream,0
    def read(self,size):
        data=self.stream.read(min(size,MAX_EXPANDED-self.total+1));self.total+=len(data)
        require(self.total<=MAX_EXPANDED,"expanded archive bound exceeded")
        return data
def extract(args):
    source=Path(args.archive).absolute()
    require(source.resolve()==source and source.is_file() and digest(source)==args.archive_sha256,"exact canonical archive required")
    wanted=Path(args.member)
    require(not wanted.is_absolute() and wanted.parts and all(p not in {"..","."} for p in wanted.parts),"exact relative member required")
    seen=set();found=False
    with source.open("rb") as raw, gzip.GzipFile(fileobj=raw) as zipped:
        with tarfile.open(fileobj=Expanded(zipped),mode="r|") as archive:
            for member in archive:
                path=Path(member.name)
                require(len(seen)<8192 and member.name not in seen and not path.is_absolute() and ".." not in path.parts,"archive member bounds/identity")
                seen.add(member.name)
                require(member.isfile() or member.isdir(),"archive links and special nodes refused")
                require(member.size<=MAX_FILE,"archive member size bound")
                if member.name!=args.member: continue
                require(member.isfile() and not found and member.size==args.byte_count,"selected member type/size differs")
                found=True;result=hashlib.sha256();count=0
                with archive.extractfile(member) as reader, output(args.output) as target:
                    while data:=reader.read(65536):
                        count+=len(data);require(count<=args.byte_count,"selected member grew");target.write(data);result.update(data)
                    require(count==args.byte_count and result.hexdigest()==args.sha256,"selected member digest differs; preserve output")
                    target.flush();os.fsync(target.fileno())
    require(found,"selected archive member missing")
    require(digest(source)==args.archive_sha256,"source archive changed during extraction")
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    commands=parser.add_subparsers(dest="action",required=True)
    acquire_parser=commands.add_parser("acquire")
    acquire_parser.add_argument("--url",required=True);acquire_parser.add_argument("--allow-network",action="store_true")
    extract_parser=commands.add_parser("extract")
    for name in ["archive","archive-sha256","member"]:extract_parser.add_argument("--"+name,required=True)
    extract_parser.add_argument("--byte-count",type=int,required=True)
    for child in [acquire_parser,extract_parser]: child.add_argument("--output",required=True);child.add_argument("--sha256",required=True)
    args=parser.parse_args()
    require(re.fullmatch(r"[0-9a-f]{64}",args.sha256),"exact expected asset SHA256 required")
    if args.action=="acquire":acquire(args)
    else:require(0<args.byte_count<=MAX_FILE,"selected member byte bound");extract(args)
if __name__=="__main__":main()
