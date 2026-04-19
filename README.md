# Typos Language Server

[Typos Language Server](https://github.com/tekumara/typos-lsp) support for Zed editor.

Typos is a spell checking tool using a list of commonly known typos, allowing it to have fewer false-positives than dictionary-based tools like [CSpell](https://github.com/streetsidesoftware/cspell).
This is a great alternative for CI/CD integration, but I would suggest using both.

## Installation

You can install this extension directly through Zed's extension marketplace.

## Issues

This extension merely downloads then launches the LSP through Zed. The Zed extension API being as thin as it is today we cannot add custom commands to interact with it directly (to toggle the LSP for specific files for example).

For issues related to code actions or the `typos-lsp` crate, please send them over to the [tekumara/typos-lsp](https://github.com/tekumara/typos-lsp) repository.

## Configuration

The Typos extension can be configured through a `.typos.toml` configuration file, which reference can be found [here](https://github.com/crate-ci/typos/blob/master/docs/reference.md).

Zed configuration for the typos-lsp server is entirely optional and only needed if you want to customise typos-lsp.
Everything under `initialization_options` is passed to the server during initialization.

The `binary` section can be used to choose the executable, pass extra argv, or set process environment variables.
See Zed’s [Configuring Languages](https://zed.dev/docs/configuring-languages) documentation.

```javascript
{
    "lsp": {
        "typos": {
            // Optional. Omit the entire "binary" object to use Zed’s default typos-lsp discovery.
            "binary": {
                // Prefer your install instead of auto-download when applicable.
                "ignore_system_version": false,
                "path": "/absolute/path/to/typos-lsp",
                "arguments": [],
                "env": {
                    // Logging level for the raw language server logs (defaults to error).
                    // Raw logs appear in the LSP Logs under Server Logs when Log level = Log
                    "RUST_LOG": "typos_lsp=error"
                }
            },
            "initialization_options": {
                // Custom config. Used together with a config file found in the workspace or its parents,
                // taking precedence for settings declared in both.
                // Equivalent to the typos `--config` cli argument.
                "config": "~/code/typos-lsp/crates/typos-lsp/tests/typos.toml",
                // Diagnostic severity within Zed. "Information" by default, can be:
                // "Error", "Hint", "Information", "Warning"
                "diagnosticSeverity": "Information",
            }
        }
    }
}
```

**WARNING**: When modifying your Typos configuration in `typos.toml` or `Cargo.toml`, you will need to reload the workspace for the changes to take effect.
