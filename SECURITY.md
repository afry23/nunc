# Security Policy

## Supported versions

Only the latest release receives security fixes. nunc is maintained by a single person; fixes are made on a best-effort
basis.

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Report privately through GitHub: [Security → Report a vulnerability](https://github.com/afry23/nunc/security/advisories/new).
Include the affected version and platform, steps to reproduce, and the impact you expect.

You can expect an answer within about a week. Once a fix is released, the advisory is published and you are credited
unless you prefer otherwise.

## Scope

Of particular interest:

- the local HTTP interface (`127.0.0.1`, bearer token, [docs/api.md](./docs/api.md), [ADR 0013](./docs/adr/0013-lokale-api.md)):
  authentication bypass, access from a browser or from other hosts, token leakage;
- handling of secrets (system keychain, [ADR 0005](./docs/adr/0005-secret-store.md)): tokens or passwords in logs,
  database, settings or backups;
- sync (WebDAV, Google Drive, Todoist) and OAuth flow: credential or data leakage, unsafe handling of remote data;
- backup and restore: path traversal, malicious database files;
- release pipeline: anything that lets an outsider influence signed artifacts.

Out of scope: issues that need a compromised device or operating system account, and findings in third-party services.
