# Zed C#

A [C#](https://learn.microsoft.com/en-us/dotnet/csharp/) extension for [Zed](https://zed.dev).

## Debugging

The extension provides the [netcoredbg](https://github.com/Samsung/netcoredbg)
debug adapter. The adapter binary is downloaded automatically on first use; to
use your own build instead:

```jsonc
{
  "dap": {
    "netcoredbg": {
      "binary": "/usr/local/bin/netcoredbg"
    }
  }
}
```

Create `.zed/debug.json` in your project for explicit configurations:

```jsonc
[
  {
    "label": "Debug console app",
    "adapter": "netcoredbg",
    "request": "launch",
    "program": "$ZED_WORKTREE_ROOT/bin/Debug/net10.0/App.dll",
    "cwd": "$ZED_WORKTREE_ROOT",
    "stopAtEntry": false,
    "justMyCode": true
  },
  {
    "label": "Attach to process",
    "adapter": "netcoredbg",
    "request": "attach",
    "processId": 0
  }
]
```

Debugging a `dotnet run` task builds the project, asks MSBuild for the
assembly it produced, and launches netcoredbg against it. Arguments after `--`
are passed to the program. Start it from a file in the project's folder.

## Runnables

`Main` methods and files of top-level statements get a run button in the
gutter.

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions) section of the Zed docs.
