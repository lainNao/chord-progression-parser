import { expect, test } from "bun:test";
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

/** Runs the real setup target with isolated tools, without installing anything on the host. */
async function runInstall({
  installed,
  installExit,
}: {
  installed: boolean;
  installExit: number;
}): Promise<{ exitCode: number; commands: string }> {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-parser-install-"),
  );
  try {
    const log = path.join(directory, "commands");
    await writeFile(log, "");
    for (const tool of [
      "cargo",
      "rustup",
      "bun",
      ...(installed ? ["wasm-pack"] : []),
    ]) {
      const executable = path.join(directory, tool);
      await writeFile(
        executable,
        `#!/bin/sh
printf '%s %s\\n' '${tool}' "$*" >> "$COMMAND_LOG"
if [ '${tool}' = cargo ] && [ "$2" = wasm-pack ]; then
  exit "$INSTALL_EXIT"
fi
`,
      );
      await chmod(executable, 0o755);
    }
    const child = Bun.spawn(
      [
        "/usr/bin/make",
        "-f",
        path.join(import.meta.dir, "../Makefile"),
        "install",
      ],
      {
        cwd: directory,
        env: {
          ...process.env,
          // Exclude the real user-installed Rust/WASM tools from the fixture.
          PATH: directory,
          COMMAND_LOG: log,
          INSTALL_EXIT: String(installExit),
        },
        stdout: "ignore",
        stderr: "ignore",
      },
    );
    const exitCode = await child.exited;
    return { exitCode, commands: await readFile(log, "utf8") };
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

/** A clean developer machine needs the wasm-pack executable as well as Rust targets. */
test("setup installs wasm-pack when it is missing", async (): Promise<void> => {
  const result = await runInstall({ installed: false, installExit: 0 });
  expect(result.exitCode).toBe(0);
  expect(result.commands).toContain(
    "cargo install typeshare-cli --version 1.13.4 --locked\n",
  );
  expect(result.commands).toContain("cargo install wasm-pack --locked\n");
  expect(result.commands).toContain("bun install --frozen-lockfile\n");
});

/** CI's prebuilt executable and an existing local installation must remain in use. */
test("setup keeps an existing wasm-pack installation", async (): Promise<void> => {
  const result = await runInstall({ installed: true, installExit: 42 });
  expect(result.exitCode).toBe(0);
  expect(result.commands).not.toContain("cargo install wasm-pack");
});

/** A failed tool installation must stop setup before installing JavaScript dependencies. */
test("setup propagates wasm-pack installation failures", async (): Promise<void> => {
  const result = await runInstall({ installed: false, installExit: 42 });
  expect(result.exitCode).not.toBe(0);
  expect(result.commands).toContain("cargo install wasm-pack --locked\n");
  expect(result.commands).not.toContain("bun install");
});
