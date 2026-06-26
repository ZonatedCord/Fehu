#!/usr/bin/env python3
"""Generate latest.json for Tauri 2 updater from signed release artifacts."""
import glob
import json
import os
import sys
from datetime import datetime, timezone


def read_sig(*patterns):
    for p in patterns:
        files = glob.glob(p)
        if files:
            return open(files[0]).read().strip()
    return ""


def main():
    sig_dir = sys.argv[1] if len(sys.argv) > 1 else "/tmp/sigs"
    tag = os.environ["TAG"]
    version = tag.lstrip("v")
    base = f"https://github.com/ZonatedCord/Fehu/releases/download/{tag}"

    manifest = {
        "version": version,
        "notes": "",
        "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.000Z"),
        "platforms": {
            "darwin-aarch64": {
                "signature": read_sig(f"{sig_dir}/Fehu_aarch64.app.tar.gz.sig"),
                "url": f"{base}/Fehu_aarch64.app.tar.gz",
            },
            "darwin-x86_64": {
                "signature": read_sig(f"{sig_dir}/Fehu_x64.app.tar.gz.sig"),
                "url": f"{base}/Fehu_x64.app.tar.gz",
            },
            "windows-x86_64": {
                "signature": read_sig(
                    f"{sig_dir}/*.nsis.zip.sig",
                    f"{sig_dir}/*-setup.exe.sig",
                    f"{sig_dir}/*.msi.zip.sig",
                ),
                "url": f"{base}/Fehu_{version}_x64-setup.exe",
            },
        },
    }

    print(json.dumps(manifest, indent=2))
    json.dump(manifest, open("/tmp/latest.json", "w"), indent=2)


if __name__ == "__main__":
    main()
