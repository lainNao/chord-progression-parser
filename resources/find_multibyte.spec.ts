import { expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

/** Runs the real source-inspection command inside an isolated source directory. */
async function findMultibyte(files: Record<string, string> | null): Promise<{
  exitCode: number;
  stdout: string;
  stderr: string;
}> {
  const directory = await mkdtemp(path.join(os.tmpdir(), "chord-multibyte-"));
  try {
    if (files !== null) {
      await mkdir(path.join(directory, "src"));
      for (const [name, content] of Object.entries(files)) {
        await writeFile(path.join(directory, "src", name), content);
      }
    }
    const child = Bun.spawn(
      [
        path.join(
          import.meta.dir,
          "../_tools/find_files_include_multibyte_characters.sh",
        ),
      ],
      {
        cwd: directory,
        env: { ...process.env, LC_ALL: "C.UTF-8" },
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

/** An inherited UTF-8 locale must not classify Japanese source as ASCII text. */
test("finds Unicode lines in filenames with spaces under a UTF-8 locale", async (): Promise<void> => {
  expect(
    await findMultibyte({
      "with spaces.rs": "// ASCII\n// 日本語😀\n",
      "binary.bin": "\0日本語\n",
    }),
  ).toEqual({
    exitCode: 0,
    stdout: "src/with spaces.rs:2:// 日本語😀\n",
    stderr: "",
  });
});

/** Both empty and ASCII-only source trees are successful searches with no matches. */
test("succeeds when there are no non-ASCII source lines", async (): Promise<void> => {
  const cases: Record<string, string>[] = [
    {},
    { "ascii.rs": "// ASCII only\n" },
  ];
  for (const files of cases) {
    expect(await findMultibyte(files)).toEqual({
      exitCode: 0,
      stdout: "",
      stderr: "",
    });
  }
});

/** Real filesystem failures must remain visible instead of being treated as empty results. */
test("preserves source inspection failures", async (): Promise<void> => {
  const result = await findMultibyte(null);
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).toBe("");
  expect(result.stderr).toContain("src");
});
