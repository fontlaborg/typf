#!/usr/bin/env python3
"""Fetch the pinned public Noto Sans fixture and verify its exact bytes."""
# this_file: scripts/fetch_smoke_font.py

import hashlib
from pathlib import Path
from urllib.request import urlopen

URL = "https://raw.githubusercontent.com/notofonts/noto-fonts/ffebf8c1ee449e544955a7e813c54f9b73848eac/hinted/ttf/NotoSans/NotoSans-Regular.ttf"
SHA256 = "b85c38ecea8a7cfb39c24e395a4007474fa5a4fc864f6ee33309eb4948d232d5"


def main():
    destination = Path("test-fonts/NotoSans-Regular.ttf")
    with urlopen(URL, timeout=30) as response:
        data = response.read()
    if hashlib.sha256(data).hexdigest() != SHA256:
        raise ValueError("Downloaded smoke font does not match the pinned SHA256")
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data)
    print(f"Verified pinned Noto Sans: {len(data)} bytes")


if __name__ == "__main__":
    main()
