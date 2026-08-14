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

Starting a debug session from a `dotnet run` task uses the bundled `dotnet`
locator: it runs the build first, asks MSBuild for the produced assembly with
`dotnet msbuild -getProperty:TargetPath`, and launches netcoredbg against it.

### Debugging a single test

The locator deliberately does not handle `dotnet test`: the test assembly is a
library driven by VSTest, and launching it directly exits without running
anything. Instead, run the **"dotnet test $ZED_SYMBOL (wait for debugger)"**
task from the gutter button on any test. The test host prints its process id
and waits; start an `attach` configuration with that id and breakpoints inside
the test are hit normally.

## Runnables

Test methods (`[Fact]`, `[Theory]`, `[Test]`, `[TestMethod]`, `[TestCase]`),
test classes (including xUnit classes, which carry no class-level attribute),
and `Main` methods get a run button in the gutter.

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions) section of the Zed docs.
