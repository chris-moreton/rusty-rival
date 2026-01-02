#!/usr/bin/env python3
"""Download Syzygy tablebases from Lichess mirror."""

import os
import sys
import urllib.request
from pathlib import Path

# 3-4-5 man tablebase files (WDL and DTZ)
# These are all the files needed for positions with up to 5 pieces
TABLEBASE_FILES = [
    # 3-man
    "KBvK.rtbw", "KBvK.rtbz",
    "KNvK.rtbw", "KNvK.rtbz",
    "KPvK.rtbw", "KPvK.rtbz",
    "KQvK.rtbw", "KQvK.rtbz",
    "KRvK.rtbw", "KRvK.rtbz",
    # 4-man
    "KBBvK.rtbw", "KBBvK.rtbz",
    "KBNvK.rtbw", "KBNvK.rtbz",
    "KBPvK.rtbw", "KBPvK.rtbz",
    "KBvKB.rtbw", "KBvKB.rtbz",
    "KBvKN.rtbw", "KBvKN.rtbz",
    "KBvKP.rtbw", "KBvKP.rtbz",
    "KNNvK.rtbw", "KNNvK.rtbz",
    "KNPvK.rtbw", "KNPvK.rtbz",
    "KNvKN.rtbw", "KNvKN.rtbz",
    "KNvKP.rtbw", "KNvKP.rtbz",
    "KPPvK.rtbw", "KPPvK.rtbz",
    "KPvKP.rtbw", "KPvKP.rtbz",
    "KQBvK.rtbw", "KQBvK.rtbz",
    "KQNvK.rtbw", "KQNvK.rtbz",
    "KQPvK.rtbw", "KQPvK.rtbz",
    "KQQvK.rtbw", "KQQvK.rtbz",
    "KQRvK.rtbw", "KQRvK.rtbz",
    "KQvKB.rtbw", "KQvKB.rtbz",
    "KQvKN.rtbw", "KQvKN.rtbz",
    "KQvKP.rtbw", "KQvKP.rtbz",
    "KQvKQ.rtbw", "KQvKQ.rtbz",
    "KQvKR.rtbw", "KQvKR.rtbz",
    "KRBvK.rtbw", "KRBvK.rtbz",
    "KRNvK.rtbw", "KRNvK.rtbz",
    "KRPvK.rtbw", "KRPvK.rtbz",
    "KRRvK.rtbw", "KRRvK.rtbz",
    "KRvKB.rtbw", "KRvKB.rtbz",
    "KRvKN.rtbw", "KRvKN.rtbz",
    "KRvKP.rtbw", "KRvKP.rtbz",
    "KRvKR.rtbw", "KRvKR.rtbz",
    # 5-man
    "KBBBvK.rtbw", "KBBBvK.rtbz",
    "KBBNvK.rtbw", "KBBNvK.rtbz",
    "KBBPvK.rtbw", "KBBPvK.rtbz",
    "KBBvKB.rtbw", "KBBvKB.rtbz",
    "KBBvKN.rtbw", "KBBvKN.rtbz",
    "KBBvKP.rtbw", "KBBvKP.rtbz",
    "KBNNvK.rtbw", "KBNNvK.rtbz",
    "KBNPvK.rtbw", "KBNPvK.rtbz",
    "KBNvKB.rtbw", "KBNvKB.rtbz",
    "KBNvKN.rtbw", "KBNvKN.rtbz",
    "KBNvKP.rtbw", "KBNvKP.rtbz",
    "KBPPvK.rtbw", "KBPPvK.rtbz",
    "KBPvKB.rtbw", "KBPvKB.rtbz",
    "KBPvKN.rtbw", "KBPvKN.rtbz",
    "KBPvKP.rtbw", "KBPvKP.rtbz",
    "KBvKBB.rtbw", "KBvKBB.rtbz",
    "KBvKBN.rtbw", "KBvKBN.rtbz",
    "KBvKBP.rtbw", "KBvKBP.rtbz",
    "KBvKNN.rtbw", "KBvKNN.rtbz",
    "KBvKNP.rtbw", "KBvKNP.rtbz",
    "KBvKPP.rtbw", "KBvKPP.rtbz",
    "KNNNvK.rtbw", "KNNNvK.rtbz",
    "KNNPvK.rtbw", "KNNPvK.rtbz",
    "KNNvKB.rtbw", "KNNvKB.rtbz",
    "KNNvKN.rtbw", "KNNvKN.rtbz",
    "KNNvKP.rtbw", "KNNvKP.rtbz",
    "KNPPvK.rtbw", "KNPPvK.rtbz",
    "KNPvKB.rtbw", "KNPvKB.rtbz",
    "KNPvKN.rtbw", "KNPvKN.rtbz",
    "KNPvKP.rtbw", "KNPvKP.rtbz",
    "KNvKBB.rtbw", "KNvKBB.rtbz",
    "KNvKBN.rtbw", "KNvKBN.rtbz",
    "KNvKBP.rtbw", "KNvKBP.rtbz",
    "KNvKNN.rtbw", "KNvKNN.rtbz",
    "KNvKNP.rtbw", "KNvKNP.rtbz",
    "KNvKPP.rtbw", "KNvKPP.rtbz",
    "KPPPvK.rtbw", "KPPPvK.rtbz",
    "KPPvKB.rtbw", "KPPvKB.rtbz",
    "KPPvKN.rtbw", "KPPvKN.rtbz",
    "KPPvKP.rtbw", "KPPvKP.rtbz",
    "KPvKBB.rtbw", "KPvKBB.rtbz",
    "KPvKBN.rtbw", "KPvKBN.rtbz",
    "KPvKBP.rtbw", "KPvKBP.rtbz",
    "KPvKNN.rtbw", "KPvKNN.rtbz",
    "KPvKNP.rtbw", "KPvKNP.rtbz",
    "KPvKPP.rtbw", "KPvKPP.rtbz",
    "KQBBvK.rtbw", "KQBBvK.rtbz",
    "KQBNvK.rtbw", "KQBNvK.rtbz",
    "KQBPvK.rtbw", "KQBPvK.rtbz",
    "KQBvKB.rtbw", "KQBvKB.rtbz",
    "KQBvKN.rtbw", "KQBvKN.rtbz",
    "KQBvKP.rtbw", "KQBvKP.rtbz",
    "KQBvKQ.rtbw", "KQBvKQ.rtbz",
    "KQBvKR.rtbw", "KQBvKR.rtbz",
    "KQNNvK.rtbw", "KQNNvK.rtbz",
    "KQNPvK.rtbw", "KQNPvK.rtbz",
    "KQNvKB.rtbw", "KQNvKB.rtbz",
    "KQNvKN.rtbw", "KQNvKN.rtbz",
    "KQNvKP.rtbw", "KQNvKP.rtbz",
    "KQNvKQ.rtbw", "KQNvKQ.rtbz",
    "KQNvKR.rtbw", "KQNvKR.rtbz",
    "KQPPvK.rtbw", "KQPPvK.rtbz",
    "KQPvKB.rtbw", "KQPvKB.rtbz",
    "KQPvKN.rtbw", "KQPvKN.rtbz",
    "KQPvKP.rtbw", "KQPvKP.rtbz",
    "KQPvKQ.rtbw", "KQPvKQ.rtbz",
    "KQPvKR.rtbw", "KQPvKR.rtbz",
    "KQQBvK.rtbw", "KQQBvK.rtbz",
    "KQQNvK.rtbw", "KQQNvK.rtbz",
    "KQQPvK.rtbw", "KQQPvK.rtbz",
    "KQQQvK.rtbw", "KQQQvK.rtbz",
    "KQQRvK.rtbw", "KQQRvK.rtbz",
    "KQQvKB.rtbw", "KQQvKB.rtbz",
    "KQQvKN.rtbw", "KQQvKN.rtbz",
    "KQQvKP.rtbw", "KQQvKP.rtbz",
    "KQQvKQ.rtbw", "KQQvKQ.rtbz",
    "KQQvKR.rtbw", "KQQvKR.rtbz",
    "KQRBvK.rtbw", "KQRBvK.rtbz",
    "KQRNvK.rtbw", "KQRNvK.rtbz",
    "KQRPvK.rtbw", "KQRPvK.rtbz",
    "KQRRvK.rtbw", "KQRRvK.rtbz",
    "KQRvKB.rtbw", "KQRvKB.rtbz",
    "KQRvKN.rtbw", "KQRvKN.rtbz",
    "KQRvKP.rtbw", "KQRvKP.rtbz",
    "KQRvKQ.rtbw", "KQRvKQ.rtbz",
    "KQRvKR.rtbw", "KQRvKR.rtbz",
    "KQvKBB.rtbw", "KQvKBB.rtbz",
    "KQvKBN.rtbw", "KQvKBN.rtbz",
    "KQvKBP.rtbw", "KQvKBP.rtbz",
    "KQvKNN.rtbw", "KQvKNN.rtbz",
    "KQvKNP.rtbw", "KQvKNP.rtbz",
    "KQvKPP.rtbw", "KQvKPP.rtbz",
    "KQvKRB.rtbw", "KQvKRB.rtbz",
    "KQvKRN.rtbw", "KQvKRN.rtbz",
    "KQvKRP.rtbw", "KQvKRP.rtbz",
    "KQvKRR.rtbw", "KQvKRR.rtbz",
    "KRBBvK.rtbw", "KRBBvK.rtbz",
    "KRBNvK.rtbw", "KRBNvK.rtbz",
    "KRBPvK.rtbw", "KRBPvK.rtbz",
    "KRBvKB.rtbw", "KRBvKB.rtbz",
    "KRBvKN.rtbw", "KRBvKN.rtbz",
    "KRBvKP.rtbw", "KRBvKP.rtbz",
    "KRBvKR.rtbw", "KRBvKR.rtbz",
    "KRNNvK.rtbw", "KRNNvK.rtbz",
    "KRNPvK.rtbw", "KRNPvK.rtbz",
    "KRNvKB.rtbw", "KRNvKB.rtbz",
    "KRNvKN.rtbw", "KRNvKN.rtbz",
    "KRNvKP.rtbw", "KRNvKP.rtbz",
    "KRNvKR.rtbw", "KRNvKR.rtbz",
    "KRPPvK.rtbw", "KRPPvK.rtbz",
    "KRPvKB.rtbw", "KRPvKB.rtbz",
    "KRPvKN.rtbw", "KRPvKN.rtbz",
    "KRPvKP.rtbw", "KRPvKP.rtbz",
    "KRPvKR.rtbw", "KRPvKR.rtbz",
    "KRRBvK.rtbw", "KRRBvK.rtbz",
    "KRRNvK.rtbw", "KRRNvK.rtbz",
    "KRRPvK.rtbw", "KRRPvK.rtbz",
    "KRRRvK.rtbw", "KRRRvK.rtbz",
    "KRRvKB.rtbw", "KRRvKB.rtbz",
    "KRRvKN.rtbw", "KRRvKN.rtbz",
    "KRRvKP.rtbw", "KRRvKP.rtbz",
    "KRRvKR.rtbw", "KRRvKR.rtbz",
    "KRvKBB.rtbw", "KRvKBB.rtbz",
    "KRvKBN.rtbw", "KRvKBN.rtbz",
    "KRvKBP.rtbw", "KRvKBP.rtbz",
    "KRvKNN.rtbw", "KRvKNN.rtbz",
    "KRvKNP.rtbw", "KRvKNP.rtbz",
    "KRvKPP.rtbw", "KRvKPP.rtbz",
    "KRvKRB.rtbw", "KRvKRB.rtbz",
    "KRvKRN.rtbw", "KRvKRN.rtbz",
    "KRvKRP.rtbw", "KRvKRP.rtbz",
]

BASE_URL = "http://tablebase.sesse.net/syzygy/3-4-5/"


def download_file(url: str, dest: Path) -> bool:
    """Download a file from URL to destination."""
    try:
        print(f"  Downloading {dest.name}...", end=" ", flush=True)
        urllib.request.urlretrieve(url, dest)
        size_mb = dest.stat().st_size / (1024 * 1024)
        print(f"OK ({size_mb:.1f} MB)")
        return True
    except Exception as e:
        print(f"FAILED: {e}")
        return False


def main():
    # Determine output directory
    script_dir = Path(__file__).parent.parent
    syzygy_dir = script_dir / "engines" / "syzygy"

    if len(sys.argv) > 1:
        syzygy_dir = Path(sys.argv[1])

    syzygy_dir.mkdir(parents=True, exist_ok=True)

    print(f"Downloading 3-4-5 man Syzygy tablebases to: {syzygy_dir}")
    print(f"Total files: {len(TABLEBASE_FILES)}")
    print()

    downloaded = 0
    skipped = 0
    failed = 0

    for filename in TABLEBASE_FILES:
        dest = syzygy_dir / filename

        if dest.exists():
            print(f"  Skipping {filename} (already exists)")
            skipped += 1
            continue

        url = BASE_URL + filename
        if download_file(url, dest):
            downloaded += 1
        else:
            failed += 1

    print()
    print(f"Done! Downloaded: {downloaded}, Skipped: {skipped}, Failed: {failed}")

    # Calculate total size
    total_size = sum((syzygy_dir / f).stat().st_size for f in TABLEBASE_FILES if (syzygy_dir / f).exists())
    print(f"Total size: {total_size / (1024 * 1024):.1f} MB")


if __name__ == "__main__":
    main()
