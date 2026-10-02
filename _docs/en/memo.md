# MEMO

The release workflows support retries after a partial publication failure.

- npm and crates.io publication check the exact package version through the shared
  [registry check](../../_tools/release/is_published.sh). Published versions are
  skipped successfully.
- Registry and network errors stop the publication job. Only HTTP 404 means the
  requested version has not been published.
- The GitHub Release is created after all package publication jobs succeed.
  An existing GitHub Release is retained when retrying the same tag.

To retry, run [test-and-release](../../.github/workflows/test-and-release.yml)
manually in GitHub Actions and specify the existing tag in `tag-to-release`.
The tag must match the version in that tag's `Cargo.toml`. See the
[development guide](./how-to-develop.md#release) for release setup.
