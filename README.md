# Zed C#

A [C#](https://learn.microsoft.com/en-us/dotnet/csharp/) extension for [Zed](https://zed.dev).

## Blazor and Razor

The extension recognizes `.razor` and `.cshtml` files as **Razor**, with a
dedicated parser for Razor directives, C# blocks, expressions, and markup.
Install Zed's HTML extension for the injected HTML syntax highlighting.

Razor files use the same Roslyn server as C# files, with the
`aspnetcorerazor` language ID required by Roslyn's Razor cohost. Use a recent
`roslyn-language-server` package that includes the built-in Razor cohost
(`Microsoft.VisualStudioCode.RazorExtension.dll` and the matching Razor compiler).
Older servers and csharp-ls/OmniSharp are not wired up for Blazor components.
If overriding `lsp.roslyn.binary.path`, supply a complete compatible server
distribution and the `--stdio`, `--autoLoadProjects` arguments.

Open the folder containing the solution or project, rather than an individual
component. C# completion, hover, and navigation in components depend on successful
project loading. HTML-specific language-server features that require Razor's
VS Code client delegation are not implemented by this extension.

## Project packages and extension methods

Language servers receive the worktree's shell environment, including
`NUGET_PACKAGES`, SDK paths, and credential-provider configuration. Project
restore and reference resolution use MSBuild/NuGet's normal `NuGet.Config`
hierarchy and package cache, independently of the server-download feed below.
The extension does not override project package sources or run a separate
restore with its own feed.

Roslyn completion includes symbols from unimported namespaces by default.
Override this with:

```json
{
  "lsp": {
    "roslyn": {
      "settings": {
        "csharp|completion": {
          "dotnet_show_completion_items_from_unimported_namespaces": false
        }
      }
    }
  }
}
```

If Roslyn reports that a restored package was not found (`NETSDK1064`), compare
the failing project's `obj/project.assets.json` `packageFolders` with the package
cache used by your working CLI. Restart Zed from the same shell in which
`dotnet build` succeeds if environment variables were changed after Zed started.
Do not change `nuget_source` to repair project-reference errors: that setting
only controls language-server installation.

## Language-server environment variables

Set `lsp.<server>.binary.env` in your Zed user settings or the workspace's
`.zed/settings.json`:

```json
{
  "lsp": {
    "roslyn": {
      "binary": {
        "env": {
          "NUGET_PACKAGES": "C:\\NuGet\\packages",
          "DOTNET_ROOT": "C:\\Program Files\\dotnet"
        }
      }
    }
  }
}
```

This works for `roslyn`, `csharp-ls`, and `omnisharp`, including custom binaries.
Zed resolves user and workspace settings; workspace values take precedence.
Configured variables override the inherited worktree shell environment
(case-insensitively on Windows), while other variables are preserved.
Values are literal strings; an empty string sets an empty value, not an unset
variable. No shell-variable expansion is performed.

Restart the language server after changing these settings. Variables affect
the server process and its children, including MSBuild and NuGet restore, not
the extension's own server downloads.

## Alternative NuGet feed

Roslyn and csharp-ls download their language-server packages from nuget.org by
default. To use a mirror instead, set `nuget_source` in Zed's user
settings or your project's `.zed/settings.json`:

```json
{
  "lsp": {
    "roslyn": {
      "settings": {
        "nuget_source": "https://packages.example.com/nuget/v3/index.json"
      }
    },
    "csharp-ls": {
      "settings": {
        "nuget_source": "https://packages.example.com/nuget/v3/index.json"
      }
    }
  }
}
```

Use the HTTP(S) **NuGet v3 service index URL**, not the package download URL.
The configured feed replaces nuget.org completely; failures do not fall back
to the public feed. The extension follows the package download endpoint advertised
by that index, so your mirror must advertise an accessible internal endpoint.
The feed must contain the selected server's packages (`roslyn-language-server.<rid>`
or `csharp-ls`). Authentication, local-directory feeds, and automatic discovery
from `NuGet.Config` are not supported.

Restart Zed after changing this setting. Already installed server versions may
be reused. This setting only controls the extension's language-server downloads,
not NuGet restore for your projects, and does not affect OmniSharp.

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions) section of the Zed docs.
