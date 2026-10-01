import { expect, test } from "bun:test";
import { chmod, mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

type RegistryScenario = {
  registry: "npm" | "crates-io";
  status: string;
  body: string;
  curlExit: number;
};

/** Runs the actual publication check against controlled HTTP and transport outcomes. */
async function runRegistryProbe({
  registry,
  status,
  body,
  curlExit,
}: RegistryScenario): Promise<{
  exitCode: number;
  stdout: string;
  stderr: string;
}> {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-parser-registry-"),
  );
  try {
    const curlPath = path.join(directory, "curl");
    await writeFile(
      curlPath,
      `#!/usr/bin/env bash
set -eu
while [ "$#" -gt 0 ]; do
  if [ "$1" = --output ]; then
    destination="$2"
    shift 2
  else
    shift
  fi
done
printf '%s' "$REGISTRY_BODY" > "$destination"
printf '%s' "$REGISTRY_STATUS"
exit "$REGISTRY_EXIT"
`,
    );
    await chmod(curlPath, 0o755);
    const systemPath = process.env.PATH;
    if (!systemPath)
      throw new Error("PATH must be set to run shell integration tests");
    const child = Bun.spawn(
      [
        "bash",
        path.join(import.meta.dir, "../_tools/release/is_published.sh"),
        registry,
        "example-package",
        "0.9.4",
      ],
      {
        env: {
          ...process.env,
          PATH: `${directory}${path.delimiter}${systemPath}`,
          REGISTRY_BODY: body,
          REGISTRY_STATUS: status,
          REGISTRY_EXIT: String(curlExit),
        },
        stdout: "pipe",
        stderr: "pipe",
      },
    );
    const [exitCode, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    return { exitCode, stdout, stderr };
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

/** Local workspace metadata must never replace verified registry publication status. */
for (const [registry, body] of [
  ["npm", JSON.stringify({ name: "example-package", version: "0.9.4" })],
  [
    "crates-io",
    JSON.stringify({ version: { crate: "example-package", num: "0.9.4" } }),
  ],
] as const) {
  test(`recognizes a published ${registry} version`, async (): Promise<void> => {
    expect(
      await runRegistryProbe({ registry, status: "200", body, curlExit: 0 }),
    ).toEqual({ exitCode: 0, stdout: "true\n", stderr: "" });
  });
}

for (const registry of ["npm", "crates-io"] as const) {
  /** A genuine 404 is the only failed HTTP response that means publication is still needed. */
  test(`recognizes an unpublished ${registry} version`, async (): Promise<void> => {
    expect(
      await runRegistryProbe({
        registry,
        status: "404",
        body: "{}",
        curlExit: 0,
      }),
    ).toEqual({ exitCode: 0, stdout: "false\n", stderr: "" });
  });

  for (const status of ["401", "429", "503"]) {
    /** Registry errors must stop release processing rather than trigger publication. */
    test(`stops ${registry} checks on HTTP ${status}`, async (): Promise<void> => {
      const result = await runRegistryProbe({
        registry,
        status,
        body: "registry failed",
        curlExit: 0,
      });
      expect(result.exitCode).not.toBe(0);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain(`HTTP ${status}`);
    });
  }

  /** Transport failures must not be mistaken for an unpublished package. */
  test(`stops ${registry} checks on transport failure`, async (): Promise<void> => {
    const result = await runRegistryProbe({
      registry,
      status: "000",
      body: "",
      curlExit: 7,
    });
    expect(result.exitCode).toBe(7);
    expect(result.stdout).toBe("");
  });

  for (const [label, body] of [
    ["non-JSON", "not JSON"],
    ["missing fields", "{}"],
    [
      "wrong version",
      JSON.stringify({ name: "example-package", version: "0.9.3" }),
    ],
  ]) {
    /** A successful status cannot hide malformed or mismatched package metadata. */
    test(`rejects ${registry} metadata with ${label}`, async (): Promise<void> => {
      const result = await runRegistryProbe({
        registry,
        status: "200",
        body,
        curlExit: 0,
      });
      expect(result.exitCode).not.toBe(0);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain("invalid metadata");
    });
  }
}
