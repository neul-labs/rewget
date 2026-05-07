"""CLI entry points that delegate to the native rewget binary."""

import os
import subprocess
import sys
from pathlib import Path


def _find_binary(name: str) -> str:
    """Find the native binary, downloading if necessary."""
    # Check alongside this package
    package_dir = Path(__file__).parent.resolve()
    binary = package_dir / name
    if binary.exists():
        return str(binary)

    # Check if already installed
    from .install import download_binary

    download_binary(package_dir)
    binary = package_dir / name
    if binary.exists():
        return str(binary)

    raise FileNotFoundError(f"Could not find or download {name}")


def main() -> None:
    """Entry point for rewget CLI."""
    binary = _find_binary("rewget.exe" if os.name == "nt" else "rewget")
    result = subprocess.run([binary, *sys.argv[1:]])
    sys.exit(result.returncode)


def main_daemon() -> None:
    """Entry point for rewgetd CLI."""
    binary = _find_binary("rewgetd.exe" if os.name == "nt" else "rewgetd")
    result = subprocess.run([binary, *sys.argv[1:]])
    sys.exit(result.returncode)
