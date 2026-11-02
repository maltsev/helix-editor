# Configuration

To override global configuration parameters, create a `config.toml` file located in your config directory:

- Linux and Mac: `~/.config/helix/config.toml`
- Windows: `%AppData%\helix\config.toml`

> 💡 You can easily open the config file by typing `:config-open` within Helix normal mode.

Example config:

```toml
theme = "onedark"

[editor]
line-number = "relative"
mouse = false

[editor.cursor-shape]
insert = "bar"
normal = "block"
select = "underline"

[editor.file-picker]
hidden = false
```

You can use a custom configuration file by specifying it with the `-c` or
`--config` command line argument, for example `hx -c path/to/custom-config.toml`.
You can reload the config file by issuing the `:config-reload` command. Alternatively, on Unix operating systems, you can reload it by sending the USR1
signal to the Helix process, such as by using the command `pkill -USR1 hx`.

Finally, you can have a `config.toml` and a `languages.toml` local to a project by putting it under a `.helix` directory in your repository.
Its settings will be merged with the configuration directory and the built-in configuration.


## Icons

Customize symbols in `config.toml` under `[icons]`. File, directory, completion,
symbol-kind, and version-control icons using patched fonts are disabled by default.
Enable the groups you want and use a font that contains the glyphs, such as a Nerd
Font. Existing UI symbols remain available without a patched font.

```toml
[icons.fs]
enable = true

[icons.kind]
enable = true

[icons.vcs]
enable = true
branch = "⌬ "

[icons.fs.file.extension]
rs = { glyph = "🦀", fg = "#ff0000", padding = { left = 1, right = 1 } }

[icons.fs.file.filename]
"Cargo.toml" = "📦 "

[icons.fs.directory]
".git" = " "

[icons.diagnostic]
warning = "▲"

[icons.ui.statusline]
separator = "│"

[icons.ui.virtual]
indentation = "╎"
wrap = "↪ "
```

An icon may be a string (including `""` to hide it) or a table with `glyph`, `fg`,
`bg`, `padding = { left = 0, right = 0 }`, `modifiers = ["bold"]`, and
`underline = { color = "#ffffff", style = "curl" }`. Colors use hexadecimal RGB.
Glyphs are limited to 28 UTF-8 bytes; explicitly configured glyphs and padding must
fit within that limit. File and directory overrides inherit unspecified glyph,
style, and padding values from their defaults. Set both padding values to zero
to remove inherited padding. Inherited padding is reduced as needed to keep the
combined glyph within 28 bytes.

| Section | Settings |
| --- | --- |
| `icons.fs` | `enable`; file icons by `file.filename` and `file.extension`; directory icons by name in `directory`, plus `default` and `open` |
| `icons.kind` | `enable`; completion and symbol names such as `function`, `struct`, `variable`, `enum_member`, `type_param`, and `color` |
| `icons.vcs` | `enable`, `branch`, `added`, `removed`, `ignored`, `modified`, `renamed`, `conflict` |
| `icons.diagnostic` | `hint`, `info`, `warning`, `error` |
| `icons.dap` | `verified`, `unverified`, `play` |
| `icons.ui` | `workspace` |
| `icons.ui.gutter` | `added`, `modified`, `removed` |
| `icons.ui.virtual` | `space`, `nbsp`, `nnbsp`, `tab`, `newline`, `tabpad`, `wrap`, `indentation`, `ruler` |
| `icons.ui.statusline` | `separator` |
| `icons.ui.indicator` | `readonly`, `modified`, `insecure` |

Exact filenames take precedence over extensions. Trusted project configuration
can override individual icons; `:config-reload` refreshes them. Theme authors can
style file icons with `icons.file.<filename>` or `icons.file.<extension>`, add
`.active` for the active buffer, and style directories with
`icons.directory.<name>`. File and directory overrides in `config.toml` take
precedence over these theme styles.

When migrating an older configuration, move `editor.statusline.separator` to
`icons.ui.statusline.separator`, `editor.whitespace.characters.*` to
`icons.ui.virtual.*`, `editor.indent-guides.character` to
`icons.ui.virtual.indentation`, and the global `editor.soft-wrap.wrap-indicator`
to `icons.ui.virtual.wrap`. Language-specific wrap indicators remain supported
in `languages.toml`.
