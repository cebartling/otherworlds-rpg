# ADR-0010: Cross-Language Formatting Standards via .editorconfig

## Status

Accepted

## Context

Otherworlds RPG is a polyglot monorepo spanning Rust, Swift, TypeScript, Svelte, HTML, CSS, JSON, TOML, YAML, and Markdown. Without a shared formatting standard, developers using different editors (VS Code, IntelliJ, Neovim, Xcode) will produce inconsistent whitespace, line endings, and indentation — causing noisy diffs, merge conflicts over formatting, and an unprofessional codebase appearance.

Language-specific formatters (rustfmt, Prettier, swift-format) handle their own languages well but do not cover configuration files (TOML, YAML, JSON) or cross-language concerns (line endings, final newlines, trailing whitespace). A universal baseline is needed.

## Decision

We use `.editorconfig` at the repository root to define formatting standards across all file types. All editors and IDEs that support EditorConfig (VS Code, IntelliJ, Neovim, Xcode, Sublime Text, and others) automatically apply these settings.

### Configuration

```editorconfig
root = true

[*]
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
charset = utf-8

[*.rs]
indent_style = space
indent_size = 4

[*.{toml,yml,yaml,json}]
indent_style = space
indent_size = 2

[*.{svelte,ts,js,html,css}]
indent_style = space
indent_size = 2

[*.swift]
indent_style = space
indent_size = 4

[*.md]
trim_trailing_whitespace = false

[Makefile]
indent_style = tab
```

### Standards

| Concern | Setting | Rationale |
|---------|---------|-----------|
| Line endings | LF (`\n`) | Cross-platform consistency. Windows CRLF causes Git noise. |
| Final newline | Always | POSIX convention. Prevents "No newline at end of file" warnings. |
| Trailing whitespace | Trimmed (except Markdown) | Clean diffs. Markdown uses trailing spaces for line breaks. |
| Charset | UTF-8 | Universal text encoding. |
| Rust/Swift indent | 4 spaces | Language community convention (rustfmt default, Swift standard). |
| TS/Svelte/Web indent | 2 spaces | Frontend community convention (Prettier default). |
| Config files indent | 2 spaces | Reduces nesting depth in TOML, YAML, JSON. |
| Makefile indent | Tabs | Required by Make syntax. |

### Relationship to language formatters

`.editorconfig` sets the baseline. Language-specific formatters (rustfmt, Prettier) may enforce additional rules within their language scope. When both apply, the language formatter takes precedence — but since our `.editorconfig` settings align with formatter defaults, conflicts are avoided.

## Consequences

### Positive

- **Consistent formatting across all file types**: Every contributor's editor produces the same whitespace, regardless of platform or editor choice.
- **Clean diffs**: No spurious whitespace changes, no line ending conflicts, no missing final newlines in diffs.
- **Zero configuration for contributors**: EditorConfig is supported natively or via widely available plugins in all major editors. Contributors do not need to manually configure their editor.
- **Covers configuration files**: TOML, YAML, and JSON files (which are not covered by language-specific formatters) get consistent formatting.

### Negative

- **Editor support varies**: While most modern editors support EditorConfig, some older or niche editors may not. Contributors using unsupported editors must manually match the settings.
- **Does not replace formatters**: `.editorconfig` handles indentation and whitespace but not code-level formatting (brace placement, import ordering, etc.). Language formatters are still needed.
- **Markdown exception**: Disabling trailing whitespace trimming for Markdown means other whitespace issues in Markdown files may go unnoticed.

### Constraints imposed

- The `.editorconfig` file must remain at the repository root with `root = true`.
- All new file types added to the project must have their indent style and size defined in `.editorconfig`.
- Contributors must use an editor that supports EditorConfig or manually adhere to the defined standards.
- All text files must end with a newline.
