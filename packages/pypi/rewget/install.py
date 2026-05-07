"""Download native rewget binaries at install time."""

import os
import platform as plat
import shutil
import sys
import tarfile
import urllib.request
import zipfile
from pathlib import Path

REPO = "neul-labs/rewget"
VERSION = "1.0.1"


def get_platform():
    system = plat.system().lower()
    machine = plat.machine().lower()

    os_map = {
        "darwin": "apple-darwin",
        "linux": "unknown-linux-gnu",
        "windows": "pc-windows-msvc",
    }

    arch_map = {
        "x86_64": "x86_64",
        "amd64": "x86_64",
        "aarch64": "aarch64",
        "arm64": "aarch64",
    }

    os_name = os_map.get(system)
    arch = arch_map.get(machine)

    if not os_name or not arch:
        raise RuntimeError(
            f"Unsupported platform: {system} {machine}. "
            "Supported: macOS (Intel/ARM), Linux (x64/ARM64), Windows (x64)"
        )

    return arch, os_name


def get_artifact_name():
    arch, os_name = get_platform()
    system = plat.system().lower()

    if system == "darwin":
        name = "rewget-macos-arm64" if arch == "aarch64" else "rewget-macos-x64"
    elif system == "linux":
        name = "rewget-linux-arm64" if arch == "aarch64" else "rewget-linux-x64"
    else:
        name = "rewget-windows-x64"

    ext = "zip" if system == "windows" else "tar.gz"
    return f"{name}.{ext}"


def download_binary(install_dir: Path) -> None:
    """Download and extract the native binary to install_dir."""
    artifact = get_artifact_name()
    url = f"https://github.com/{REPO}/releases/download/v{VERSION}/{artifact}"
    temp_file = install_dir / artifact

    rewget_bin = "rewget.exe" if plat.system().lower() == "windows" else "rewget"
    rewgetd_bin = "rewgetd.exe" if plat.system().lower() == "windows" else "rewgetd"

    if (install_dir / rewget_bin).exists():
        return

    print(f"Downloading rewget v{VERSION} for {plat.system()} {plat.machine()}...")
    print(f"  {url}")

    req = urllib.request.Request(
        url,
        headers={"User-Agent": "rewget-pypi-installer"},
    )

    try:
        with urllib.request.urlopen(req) as response:
            with open(temp_file, "wb") as f:
                shutil.copyfileobj(response, f)

        if artifact.endswith(".zip"):
            with zipfile.ZipFile(temp_file, "r") as z:
                for member in z.namelist():
                    # Strip leading directory if present
                    parts = member.split("/", 1)
                    if len(parts) > 1 and parts[1]:
                        z.extract(member, install_dir)
                        extracted = install_dir / member
                        target = install_dir / parts[1]
                        if extracted != target:
                            extracted.rename(target)
        else:
            with tarfile.open(temp_file, "r:gz") as tar:
                for member in tar.getmembers():
                    parts = member.name.split("/", 1)
                    if len(parts) > 1 and parts[1]:
                        member.name = parts[1]
                        tar.extract(member, install_dir)

        temp_file.unlink(missing_ok=True)

        # Make executable on Unix
        if plat.system().lower() != "windows":
            (install_dir / rewget_bin).chmod(0o755)
            (install_dir / rewgetd_bin).chmod(0o755)

        print("rewget installed successfully.")
    except Exception as e:
        if temp_file.exists():
            temp_file.unlink(missing_ok=True)
        raise RuntimeError(f"Failed to install rewget: {e}")
