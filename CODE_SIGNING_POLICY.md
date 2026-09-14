# Code signing policy

## Scope

Only official Mistria Tracker release artifacts built from this repository's
`main` branch may be submitted for code signing. Local, modified, or
third-party builds are never submitted as official releases.

## Roles

This is a solo-maintained project. The repository owner, `DarkySpear5`, acts as
the committer, reviewer, and release approver. Each signing request is manually
reviewed and approved only after the public source, build scripts, and test
results have been checked.

## Release process

1. Build the desktop executable and installer from the public source on a
   GitHub-hosted build runner.
2. Run the test suite and publish the unsigned build as a workflow artifact.
3. Manually approve the signing request for a versioned release from `main`.
4. Publish only the returned signed artifact and its SHA-256 checksum.

## Privacy and safety

Mistria Tracker does not transfer information to other networked systems unless
the user explicitly requests it. It stores tracker data locally and never
modifies Fields of Mistria save files. See [PRIVACY.md](PRIVACY.md).

## Signing provider

Pending approval, free code signing will be provided by
[SignPath.io](https://about.signpath.io/), with the certificate issued by
[SignPath Foundation](https://signpath.org/).
