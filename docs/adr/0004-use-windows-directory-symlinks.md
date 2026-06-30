# Use Windows directory symlinks for dock mode

Windows dock mode uses real directory symbolic links via the platform symlink API. This keeps the `symlink` mode and the symlink dock glossary literal across supported platforms.

Junctions were rejected because they would make Windows `symlink` mode behave differently from its name and documentation. A silent fallback to folder mode was also rejected because dock mode exists to give Zed one visible root where project entries are links.

Creating directory symbolic links on Windows can require Developer Mode, `SeCreateSymbolicLinkPrivilege`, or an administrator shell. When link creation fails, the CLI should fail with an actionable error instead of changing modes implicitly.
