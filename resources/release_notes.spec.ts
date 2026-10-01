import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

/** Runs shell commands inside an isolated repository with a synthetic author. */
async function runScript({
  directory,
  script,
  previousTag,
}: {
  directory: string;
  script: string;
  previousTag: string;
}): Promise<void> {
  // Git hooks can export GIT_DIR or GIT_INDEX_FILE for the caller's repository.
  const environment = Object.fromEntries(
    Object.entries(process.env).filter(([name]) => !name.startsWith("GIT_")),
  );
  const child = Bun.spawn(
    ["bash", "-e", "-u", "-o", "pipefail", "-c", script],
    {
      cwd: directory,
      env: {
        ...environment,
        GIT_CONFIG_NOSYSTEM: "1",
        GIT_CONFIG_GLOBAL: "/dev/null",
        GIT_AUTHOR_NAME: "Release test",
        GIT_AUTHOR_EMAIL: "release-test@example.invalid",
        GIT_COMMITTER_NAME: "Release test",
        GIT_COMMITTER_EMAIL: "release-test@example.invalid",
        PREV_TAG: previousTag,
        CHANGE_COUNT: previousTag === "" ? "47" : "1",
        NEW_TAG: "v0.9.4",
        GITHUB_OUTPUT: path.join(directory, "workflow-output"),
        FEATURE_SUBJECT:
          "feat: support $(touch unexpected-command) and backticks `literal`",
      },
      stdout: "ignore",
      stderr: "pipe",
    },
  );
  const [exitCode, stderr] = await Promise.all([
    child.exited,
    new Response(child.stderr).text(),
  ]);
  if (exitCode !== 0) throw new Error(`Release script failed: ${stderr}`);
}

/** Exercises the real release-note step without a remote or publication credentials. */
async function createReleaseNotes(previousTag: string): Promise<string> {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-parser-notes-"),
  );
  try {
    await runScript({
      directory,
      previousTag,
      script: `
git init --quiet
git -c core.hooksPath=/dev/null commit --quiet --allow-empty -m 'feat: initial feature'
git tag v0.9.3
for ((index = 1; index <= CHANGE_COUNT; index++)); do
  git -c core.hooksPath=/dev/null commit --quiet --allow-empty -m "fix: change $index"
done
git -c core.hooksPath=/dev/null commit --quiet --allow-empty -m 'docs: explain notation'
git -c core.hooksPath=/dev/null commit --quiet --allow-empty -m "$FEATURE_SUBJECT"
git -c core.hooksPath=/dev/null commit --quiet --allow-empty -m 'chore: release 0.9.4'
`,
    });
    const workflow = Bun.YAML.parse(
      await readFile(
        path.join(import.meta.dir, "../.github/workflows/test-and-release.yml"),
        "utf8",
      ),
    ) as {
      jobs: Record<string, { steps: { name?: string; run?: string }[] }>;
    };
    const step = workflow.jobs["test-and-release"].steps.find(
      (candidate) => candidate.name === "Prepare release",
    );
    if (!step?.run) throw new Error("Release preparation step was not found");
    await runScript({
      directory,
      previousTag,
      // A manual retry generates notes without pushing a tag from the test repository.
      script: step.run.replaceAll(
        "${{ github.event_name }}",
        "workflow_dispatch",
      ),
    });
    expect(
      await Bun.file(path.join(directory, "unexpected-command")).exists(),
    ).toBe(false);
    return await readFile(path.join(directory, "workflow-output"), "utf8");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

/** An initial release must include the entire history, including changes older than 50 commits. */
test("first release notes contain every commit", async (): Promise<void> => {
  const notes = await createReleaseNotes("");
  expect(notes).toContain("feat: initial feature");
  expect(notes.match(/^- /gm)).toHaveLength(51);
}, 30_000);

/** Later releases include every commit type since the previous tag, with subjects treated as text. */
test("release notes include features and docs beyond the release commit", async (): Promise<void> => {
  const notes = await createReleaseNotes("v0.9.3");
  expect(notes).not.toContain("feat: initial feature");
  expect(notes).toContain("docs: explain notation");
  expect(notes).toContain(
    "feat: support $(touch unexpected-command) and backticks `literal`",
  );
  expect(notes).toContain("chore: release 0.9.4");
  expect(notes.match(/^- /gm)).toHaveLength(4);
}, 30_000);
