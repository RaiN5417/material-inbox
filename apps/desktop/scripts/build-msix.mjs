// Runs the MSIX build with an *absolute* CARGO_TARGET_DIR.
//
// This is a Cargo workspace, so `cargo build` always writes to the
// workspace-root `target/`, not `src-tauri/target/`. tauri-windows-bundle
// assumes the latter unless CARGO_TARGET_DIR says otherwise, so we point it
// there explicitly. It has to be absolute: tauri-cli invokes cargo from
// `src-tauri`, one directory deeper than this script's own cwd
// (`apps/desktop`), so a relative value resolves to two different
// directories depending on which process reads it.
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const desktopDir = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const targetDir = path.resolve(desktopDir, "../../target");

// shell: true is required on Windows to resolve the pnpm.cmd shim.
execFileSync("pnpm", ["exec", "tauri-windows-bundle", "build", "--runner", "pnpm", ...process.argv.slice(2)], {
  cwd: desktopDir,
  stdio: "inherit",
  shell: true,
  env: { ...process.env, CARGO_TARGET_DIR: targetDir },
});
