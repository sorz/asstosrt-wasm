#!/usr/bin/env python3
"""Append hash to web worker's filename after build."""

from pathlib import Path
from hashlib import blake2b, file_digest
import re
import sys


DIST = Path("dist/.stage")
WORKER_WASM = DIST / "worker_bg.wasm"
WOEKRE_JS = DIST / "worker.js"
WORKER_LD = DIST / "worker_loader.js"
INDEX = DIST / "index.html"

HASH_SIZE_IN_BYTES = 8
WORKER_LD_ORIG_HASH = "6b0b56b4fa32f063"
# ^ _file_hash("import init from './worker.js';await init();")


def _file_hash(path: Path) -> str:
    hasher = blake2b(digest_size=HASH_SIZE_IN_BYTES)
    with path.open("rb") as f:
        hash = file_digest(f, lambda: hasher)  # type: ignore
    return hash.hexdigest()


def _hash_and_rename(file: Path) -> Path:
    hash = _file_hash(file)
    new_filename = f"{file.stem}-{hash}{file.suffix}"
    return file.rename(file.parent / new_filename)


def main():
    # Rename WORKER_WASM & WOEKRE_JS
    worker_wasm = _hash_and_rename(WORKER_WASM).name
    worker_js = _hash_and_rename(WOEKRE_JS).name

    # Update URL inside WORKER_LD
    ld_hash = _file_hash(WORKER_LD)
    if ld_hash != WORKER_LD_ORIG_HASH:
        print("_file_hash({WORKER_LD}) is {ld_hash}", file=sys.stderr)
        sys.exit(1)
    with WORKER_LD.open("w") as f:
        f.write(f"import init from './{worker_js}';await init('./{worker_wasm}');")
    worker_ld = _hash_and_rename(WORKER_LD).name

    # Update index.html
    with INDEX.open("r+") as f:
        html = f.read()
        new_html = re.sub(
            r'data-worker-uri="([\w\.\/]+\.js)"',
            f'data-worker-uri="./{worker_ld}"',
            html,
            count=1,
        )
        f.seek(0)
        f.write(new_html)
        f.truncate()


if __name__ == "__main__":
    main()
