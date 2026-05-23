# Zed C#

A [C#](https://learn.microsoft.com/en-us/dotnet/csharp/) extension for [Zed](https://zed.dev).

## Language servers

This extension supports two language servers:

- **Roslyn** (default): Microsoft's `Microsoft.CodeAnalysis.LanguageServer`, downloaded from NuGet on first launch.
- **OmniSharp**: the community-maintained `OmniSharp/omnisharp-roslyn` server, downloaded from GitHub releases on first launch.

To pin a specific server, set `language_servers` in your Zed settings:

```json
{
  "languages": {
    "CSharp": {
      "language_servers": ["omnisharp", "!roslyn"]
    }
  }
}
```

## Troubleshooting

### OmniSharp fails to start on macOS Apple Silicon

If OmniSharp fails on first launch with an error like:

```
Failed to load /usr/local/share/dotnet/x64/host/fxr/<version>/libhostfxr.dylib,
error: mach-o file, but is an incompatible architecture
(have 'x86_64', need 'arm64e' or 'arm64')
```

the bundled OmniSharp binary itself is arm64-native, but the .NET host resolver is picking up a legacy x86_64 `libhostfxr.dylib` from `/usr/local/share/dotnet/x64/` (left over from an Intel/Rosetta-era installer).

Fix: install the arm64 .NET runtime and point `DOTNET_ROOT` at it explicitly in your shell init (`~/.zshrc`, `~/.zprofile`, or equivalent):

```sh
export DOTNET_ROOT="/usr/local/share/dotnet"
```

Restart Zed so it inherits the variable from the shell, then reopen the C# project.

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions) section of the Zed docs.
