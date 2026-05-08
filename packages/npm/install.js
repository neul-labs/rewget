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
  };

  const archMap = {
    x64: "x86_64",
    arm64: "aarch64",
  };

  const os = osMap[platform];
  const mappedArch = archMap[arch];

  if (!os || !mappedArch) {
    console.error(`Unsupported platform: ${platform} ${arch}`);
    console.error("rewget supports: macOS (Intel/ARM), Linux (x64/ARM64)");
    process.exit(1);
  }

  return { arch: mappedArch, os };
}

function getArtifactName() {
  const { arch, os } = getPlatform();

  if (process.platform === "darwin") {
    return os === "apple-darwin" && arch === "x86_64"
      ? `rewget-macos-x64.tar.gz`
      : `rewget-macos-arm64.tar.gz`;
  }
  return arch === "x86_64"
    ? `rewget-linux-x64.tar.gz`
    : `rewget-linux-arm64.tar.gz`;
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

  const rewgetBin = "rewget";
  const rewgetdBin = "rewgetd";

  if (fs.existsSync(path.join(binDir, rewgetBin))) {
    console.log("rewget binaries already installed.");
    return;
  }

  console.log(`Downloading rewget v${VERSION} for ${process.platform} ${process.arch}...`);
  console.log(`  ${url}`);

  try {
    await downloadFile(url, tempFile);

    execSync(`tar -xzf "${tempFile}" -C "${binDir}" --strip-components=1`, {
      stdio: "inherit",
    });

    fs.unlinkSync(tempFile);

    fs.chmodSync(path.join(binDir, rewgetBin), 0o755);
    fs.chmodSync(path.join(binDir, rewgetdBin), 0o755);

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
