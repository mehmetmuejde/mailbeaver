// Sets the app version everywhere it is declared: package.json (read by tauri.conf.json),
// the workspace Cargo.toml and the workspace packages in Cargo.lock.
// Usage: pnpm release:version 1.2.3
import { readFileSync, writeFileSync } from "node:fs";
import process from "node:process";

const version = process.argv[2];
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version ?? "")) {
  process.stderr.write("Usage: pnpm release:version <major.minor.patch>\n");
  process.exit(1);
}

function update(path, pattern, replacement) {
  const before = readFileSync(path, "utf8");
  const after = before.replace(pattern, replacement);
  if (after === before) {
    process.stderr.write(`No version found in ${path}\n`);
    process.exit(1);
  }
  writeFileSync(path, after);
}

update("package.json", /("version":\s*")[^"]+(")/, `$1${version}$2`);
update("Cargo.toml", /^(version\s*=\s*")[^"]+(")/m, `$1${version}$2`);
for (const name of ["mailbeaver", "mailbeaver-core", "mailbeaver-server"]) {
  update(
    "Cargo.lock",
    new RegExp(`(\\[\\[package\\]\\]\\nname = "${name}"\\nversion = ")[^"]+(")`),
    `$1${version}$2`,
  );
}
process.stdout.write(`Version set to ${version}. Commit, then tag and push: git tag v${version}\n`);
