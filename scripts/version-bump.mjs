#!/usr/bin/env node

/**
 * Version bump script that syncs version across:
 * - package.json
 * - src-tauri/Cargo.toml
 * - src-tauri/tauri.conf.json
 *
 * Usage:
 *   node scripts/version-bump.mjs 1.2.0
 *   node scripts/version-bump.mjs major|minor|patch
 */

import { readFileSync, writeFileSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf-8"));
}

function writeJson(path, data) {
  writeFileSync(path, JSON.stringify(data, null, 2) + "\n");
}

function parseSemver(version) {
  const match = version.match(
    /^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.]+))?$/
  );
  if (!match) return null;
  return {
    major: parseInt(match[1]),
    minor: parseInt(match[2]),
    patch: parseInt(match[3]),
    prerelease: match[4] || null,
  };
}

function bumpVersion(current, type) {
  const parsed = parseSemver(current);
  if (!parsed) throw new Error(`Invalid current version: ${current}`);

  switch (type) {
    case "major":
      return `${parsed.major + 1}.0.0`;
    case "minor":
      return `${parsed.major}.${parsed.minor + 1}.0`;
    case "patch":
      return `${parsed.major}.${parsed.minor}.${parsed.patch + 1}`;
    default:
      throw new Error(`Invalid bump type: ${type}`);
  }
}

// Main
const arg = process.argv[2];
if (!arg) {
  console.error("Usage: version-bump.mjs <version|major|minor|patch>");
  process.exit(1);
}

const pkgPath = resolve(root, "package.json");
const cargoPath = resolve(root, "src-tauri/Cargo.toml");
const tauriConfPath = resolve(root, "src-tauri/tauri.conf.json");

const pkg = readJson(pkgPath);

let newVersion;
if (["major", "minor", "patch"].includes(arg)) {
  newVersion = bumpVersion(pkg.version, arg);
} else {
  if (!parseSemver(arg)) {
    console.error(`Invalid semver version: ${arg}`);
    process.exit(1);
  }
  newVersion = arg;
}

// Update package.json
pkg.version = newVersion;
writeJson(pkgPath, pkg);
console.log(`package.json: ${newVersion}`);

// Update Cargo.toml
let cargoContent = readFileSync(cargoPath, "utf-8");
cargoContent = cargoContent.replace(
  /^version\s*=\s*"[^"]*"/m,
  `version = "${newVersion}"`
);
writeFileSync(cargoPath, cargoContent);
console.log(`Cargo.toml: ${newVersion}`);

// Update tauri.conf.json
const tauriConf = readJson(tauriConfPath);
tauriConf.version = newVersion;
writeJson(tauriConfPath, tauriConf);
console.log(`tauri.conf.json: ${newVersion}`);

console.log(`\nVersion bumped to ${newVersion}`);
