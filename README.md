# Zed C# Plus

A [C#](https://learn.microsoft.com/en-us/dotnet/csharp/) and
[Razor](https://learn.microsoft.com/en-us/aspnet/core/mvc/views/razor) extension
for [Zed](https://zed.dev), covering C#, Razor (`.cshtml` / `.razor`), and .NET
project files.

## Provenance

This is a fork of [zed-extensions/csharp](https://github.com/zed-extensions/csharp)
at commit `88597e1` (version 1.2.2), which remains under the Apache-2.0 licence
reproduced in [LICENSE](LICENSE). Original authorship is retained in
`extension.toml`. Modifications made in this fork are tracked in the Git history
and summarised per release in the changelog.

## Relationship to the upstream C# extension

C# Plus is a **superset** of the upstream extension, not a companion to it. It
registers the same file suffixes (`.cs`, `.csproj`, `.slnx`, MSBuild files), so
installing both will produce duplicate language ownership. Install one or the
other:

1. Uninstall the **C#** extension from Zed's extension list.
2. Install **C# Plus**.

Your existing `lsp.omnisharp` / `lsp.roslyn` / `lsp.csharp-ls` settings continue
to work unchanged.

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions)
section of the Zed docs.

### Staying current with upstream

This fork tracks two remotes: `origin` is
[joeizang/zed-csharp-plus](https://github.com/joeizang/zed-csharp-plus),
`upstream` is [zed-extensions/csharp](https://github.com/zed-extensions/csharp).

```sh
git fetch upstream && git merge upstream/main
```

See `BACKLOG.md`, item D0.1, for the upstream-sync policy.
