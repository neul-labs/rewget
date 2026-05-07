#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const REPO = "neul-labs/rewget";
const VERSION = require("./package.json").version;

function getPlatform() {
  const platform = process.platform;
  const arch = process.arch;

  const osMap = {
    darwin: "apple-darwin",
    linux: "unknown-linux-gnu",
    win32: "pc-windows-msvc",
  };

  const archMap = {
    x64: "x86_64",
    arm64: "aarch64",
  };

  const os = osMap[platform];
  const mappedArch = archMap[arch];

  if (!os || !mappedArch) {
    console.error(`Unsupported platform: ${platform} ${arch}`);
    console.error("rewget supports: macOS (Intel/ARM), Linux (x64/ARM64), Windows (x64)");
    process.exit(1);
  }

  return { arch: mappedArch, os };
}

function getArtifactName() {
  const { arch, os } = getPlatform();
  const ext = process.platform === "win32" ? "zip" : "tar.gz";

  if (process.platform === "darwin") {
    return os === "apple-darwin" && arch === "x86_64"
      ? `rewget-macos-x64.${ext}`
      : `rewget-macos-arm64.${ext}`;
  }
  if (process.platform === "linux") {
    return arch === "x86_64"
      ? `rewget-linux-x64.${ext}`
      : `rewget-linux-arm64.${ext}`;
  }
  return `rewget-windows-x64.${ext}`;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, { headers: { "User-Agent": "rewget-npm-installer" } }, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          downloadFile(response.headers.location, dest).then(resolve).catch(reject);
          return;
        }
        if (response.statusCode !== 200) {
          reject(new Error(`Download failed with status ${response.statusCode}`));
          return;
        }
        response.pipe(file);
        file.on("finish", () => {
          file.close(resolve);
        });
      })
      .on("error", (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
  });
}

async function install() {
  const binDir = path.join(__dirname, "bin");
  const artifact = getArtifactName();
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${artifact}`;
  const tempFile = path.join(__dirname, artifact);

  // Check if binaries already exist
  const rewgetBin = process.platform === "win32" ? "rewget.exe" : "rewget";
  const rewgetdBin = process.platform === "win32" ? "rewgetd.exe" : "rewgetd";

  if (fs.existsSync(path.join(binDir, rewgetBin))) {
    console.log("rewget binaries already installed.");
    return;
  }

  console.log(`Downloading rewget v${VERSION} for ${process.platform} ${process.arch}...`);
  console.log(`  ${url}`);

  try {
    await downloadFile(url, tempFile);

    if (artifact.endsWith(".zip")) {
      // Windows: use PowerShell Expand-Archive
      const psCmd = `Expand-Archive -Path "${tempFile}" -DestinationPath "${binDir}" -Force`;
      execSync(psCmd, { shell: "powershell.exe", stdio: "inherit" });
    } else {
      // Unix: tar extract
      execSync(`tar -xzf "${tempFile}" -C "${binDir}" --strip-components=1`, {
        stdio: "inherit",
      });
    }

    fs.unlinkSync(tempFile);

    // Make binaries executable on Unix
    if (process.platform !== "win32") {
      fs.chmodSync(path.join(binDir, rewgetBin), 0o755);
      fs.chmodSync(path.join(binDir, rewgetdBin), 0o755);
    }

    console.log("rewget installed successfully.");
  } catch (err) {
    console.error(`Failed to install rewget: ${err.message}`);
    if (fs.existsSync(tempFile)) {
      fs.unlinkSync(tempFile);
    }
    process.exit(1);
  }
}

install();
