import { expect, test } from "bun:test";
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

type WorkflowStep = {
  name?: string;
  uses?: string;
  run?: string;
  with?: Record<string, string>;
};

type Workflow = {
  jobs: Record<
    string,
    { steps: WorkflowStep[]; strategy?: { matrix: { target: string[] } } }
  >;
};

/** Reads the real publication commands so tests cannot drift to a separate script copy. */
async function getReleaseJob(
  registry: "npm" | "crates-io",
): Promise<Workflow["jobs"][string]> {
  const source = await readFile(
    path.join(
      import.meta.dir,
      `../.github/workflows/release-to-${registry}.yml`,
    ),
    "utf8",
  );
  const workflow = Bun.YAML.parse(source) as Workflow;
  const jobs = Object.values(workflow.jobs);
  if (jobs.length !== 1)
    throw new Error("Expected one publication job per release workflow");
  return jobs[0];
}

/** Runs a publication step with fake CLIs and a controlled publication-check result. */
async function runPublicationStep({
  registry,
  published,
  checkExit,
}: {
  registry: "npm" | "crates-io";
  published: boolean;
  checkExit: number;
}): Promise<{ exitCode: number; publications: string }> {
  const directory = await mkdtemp(
    path.join(os.tmpdir(), "chord-parser-publication-"),
  );
  try {
    const publicationLog = path.join(directory, "publications");
    await writeFile(publicationLog, "");
    await writeFile(
      path.join(directory, "chord-parser-is-published.sh"),
      `#!/usr/bin/env bash
printf '%s\\n' "$PUBLISHED"
exit "$CHECK_EXIT"
`,
    );
    const metadata = JSON.stringify({
      packages: [{ name: "chord-progression-parser", version: "0.9.4" }],
    });
    const cliSource = `#!/usr/bin/env bash
set -eu
case "$1" in
  metadata) printf '%s' '$METADATA' ;;
  info) exit 0 ;; # Reproduce cargo info succeeding for local workspace metadata.
  pkg)
    case "$3" in
      name) echo '\"chord-progression-parser\"' ;;
      version) echo '\"0.9.4\"' ;;
      *) exit 2 ;;
    esac
    ;;
  publish) printf '%s\\n' publish >> "$PUBLICATION_LOG" ;;
  *) exit 2 ;;
esac
`.replace("$METADATA", metadata);
    for (const cli of ["cargo", "npm"]) {
      const cliPath = path.join(directory, cli);
      await writeFile(cliPath, cliSource);
      await chmod(cliPath, 0o755);
    }
    const job = await getReleaseJob(registry);
    const step = job.steps.find(
      (candidate) =>
        candidate.name?.toLowerCase() === "publish if not already published",
    );
    if (!step?.run)
      throw new Error(
        "Expected the publication step to contain shell commands",
      );
    const systemPath = process.env.PATH;
    if (!systemPath)
      throw new Error("PATH must be set to run shell integration tests");
    const child = Bun.spawn(
      ["bash", "-e", "-u", "-o", "pipefail", "-c", step.run],
      {
        cwd: directory,
        env: {
          ...process.env,
          PATH: `${directory}${path.delimiter}${systemPath}`,
          RUNNER_TEMP: directory,
          PUBLISHED: String(published),
          CHECK_EXIT: String(checkExit),
          PUBLICATION_LOG: publicationLog,
        },
        stdout: "ignore",
        stderr: "ignore",
      },
    );
    const exitCode = await child.exited;
    return { exitCode, publications: await readFile(publicationLog, "utf8") };
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

/** Verifies skip, publish, and stop behavior without invoking a real package publication. */
test("publication steps respect verified registry status and stop on check failures", async (): Promise<void> => {
  for (const registry of ["npm", "crates-io"] as const) {
    expect(
      await runPublicationStep({ registry, published: true, checkExit: 0 }),
    ).toEqual({ exitCode: 0, publications: "" });
    expect(
      await runPublicationStep({ registry, published: false, checkExit: 0 }),
    ).toEqual({ exitCode: 0, publications: "publish\n" });
    expect(
      await runPublicationStep({ registry, published: false, checkExit: 7 }),
    ).toEqual({ exitCode: 7, publications: "" });
  }
});

/** Preserves current release tooling before checkout removes files absent from an older tag. */
test("old-tag retries retain current publication tooling and cover all npm targets", async (): Promise<void> => {
  for (const registry of ["npm", "crates-io"] as const) {
    const job = await getReleaseJob(registry);
    const checkouts = job.steps.filter((step) =>
      step.uses?.startsWith("actions/checkout@"),
    );
    expect(checkouts.map((step) => step.with?.ref)).toEqual([
      "${{ github.workflow_sha }}",
      "${{ inputs.tag-to-release }}",
    ]);
    const preserveIndex = job.steps.findIndex(
      (step) => step.name === "Preserve publication check",
    );
    expect(preserveIndex).toBeGreaterThan(job.steps.indexOf(checkouts[0]));
    expect(preserveIndex).toBeLessThan(job.steps.indexOf(checkouts[1]));
    expect(job.steps[preserveIndex].run).toContain(
      "$RUNNER_TEMP/chord-parser-is-published.sh",
    );
  }
  const npmJob = await getReleaseJob("npm");
  expect(npmJob.strategy?.matrix.target).toEqual(["bundler", "web", "node"]);
  expect(npmJob.steps.find((step) => step.name === "build")?.run).not.toContain(
    "generate-diagnostic-codes",
  );
});
