# Disclaimer

## Warranty Disclaimer

**null-e** is provided "as-is" without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose, and noninfringement.

## Limitation of Liability

In no event shall the authors or copyright holders be liable for any claim, damages, or other liability arising from the use of this software. This includes but is not limited to:

- Loss of data, files, or directories
- Corruption of project files or repositories
- Loss of build artifacts needed for deployment
- Failure of the trash/recovery mechanism
- Unintended deletion of files outside the intended scope

## Use at Your Own Risk

You are solely responsible for:

- **Maintaining backups** of your data before running any cleanup operations
- **Reviewing scan results** before confirming deletion
- **Using dry-run mode** (`--dry-run` / `-n`) to preview what will be deleted
- **Understanding what each artifact type is** before selecting it for deletion

## Trash Limitations

While null-e uses system trash by default (recoverable), be aware that trash may not work reliably in all scenarios:

- **Network drives** — trash support varies by protocol and server
- **External volumes** — some filesystems do not support trash
- **Low disk space** — moving large artifacts to trash requires temporary disk space
- **Permissions** — trash may fail silently for files owned by other users or system processes

## Developer Tool Notice

null-e is designed for software developers who understand the build artifacts, caches, and dependencies being cleaned. If you are unsure what a detected item is, **do not delete it** — use dry-run mode to investigate first.

## Recommended Safety Practices

```bash
# Always start with a dry run
null-e clean ~/projects --dry-run

# Use trash mode (the default) for recoverable deletion
null-e clean ~/projects --method trash

# Use strict git protection for important codebases
null-e clean ~/projects --protection block
```
