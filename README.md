# clap-discovery

Self-discovery for [clap](https://docs.rs/clap) CLIs: print a command **tree**
or a flat **list of command paths** so people and agents can find what a tool does.

Reusable utilities for inspecting clap commands and arguments. No async runtime,
application configuration, or application-specific error types required.

## Add to a project

Once published to crates.io:

```sh
cargo add clap-discovery
cargo add clap
```

For local development before publication, use
`cargo add clap-discovery --path /path/to/clap-discovery` instead.

The default feature set is empty: builder-only consumers do not need proc macros.
`Discovery` remains available as a plain format enum without any features.

## Drop-in `self tree` and `self list`

Enable the optional derive integration:

```sh
cargo add clap-discovery --features derive
cargo add clap --features derive
```

```rust
use clap::{CommandFactory, Parser, Subcommand};
use clap_discovery::Discovery;

#[derive(Parser)]
#[command(name = "app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover available commands
    #[command(name = "self")]
    Discover {
        #[command(subcommand)]
        format: Discovery,
    },
    /// Check application status
    Status,
}

fn main() -> std::io::Result<()> {
    match Cli::parse().command {
        Commands::Discover { format } => format.print(&Cli::command()),
        Commands::Status => Ok(()), // Your application logic here.
    }
}
```

With the `derive` feature, `Discovery` derives `clap::Subcommand` and exposes
`Tree` and `List` (`ls` alias).
You choose the enclosing subcommand name; it need not be `self`.
Dispatch discovery **before** loading configuration, opening a database, or
authenticating, so agents can inspect the CLI without side effects.

## Render or write directly

Works with both clap's builder and derive APIs:

```rust
use clap::Command;
use clap_discovery::{Discovery, list, tree};

let command = Command::new("app")
    .subcommand(Command::new("status").about("Check status"));

assert_eq!(tree(&command), "app\n└── status - Check status\n");
assert_eq!(list(&command), "app\napp status - Check status\n");

let mut buffer = Vec::new();
Discovery::Tree.write(&command, &mut buffer).unwrap();
```

- `tree(&Command)` and `list(&Command)` return plain-text strings.
- `Discovery::render` selects either format.
- `Discovery::write` accepts any `std::io::Write`.
- `Discovery::print` writes to locked stdout and returns I/O errors.

## Customize output

Use a reusable `Renderer` with consuming builder methods:

```rust
use clap::Command;
use clap_discovery::{Discovery, OptionStyle, Renderer, SortOrder, TreeStyle};

let command = Command::new("app")
    .subcommand(Command::new("status").about("Check status"));
let renderer = Renderer::new()
    .tree_style(TreeStyle::Ascii)
    .sort_order(SortOrder::Declaration)
    .option_style(OptionStyle::Short)
    .include_hidden(false)
    .show_descriptions(false)
    .show_aliases(true)
    .show_arguments(true);

assert_eq!(renderer.tree(&command), "app\n`-- status\n");
let mut buffer = Vec::new();
renderer.write(&command, Discovery::List, &mut buffer).unwrap();
```

| Builder method | Default | Controls |
| --- | --- | --- |
| `tree_style` | `TreeStyle::Unicode` | Unicode or ASCII tree connectors |
| `sort_order` | `SortOrder::Alphabetical` | Alphabetical or declaration order for sibling commands |
| `option_style` | `OptionStyle::Long` | Prefer long or short option names, with fallback |
| `include_hidden` | `false` | Hidden commands and arguments |
| `show_descriptions` | `true` | One-line command descriptions |
| `show_aliases` | `true` | Visible command aliases |
| `show_arguments` | `true` | Argument synopses |

`Renderer` also provides `list`, `render`, `write`, and `print`.
The free functions and `Discovery` methods retain their default behavior.

## Example

```sh
cargo run --features derive --example discover -- self tree
cargo run --features derive --example discover -- self list
cargo run --features derive --example discover -- self ls
```

Tree:

```text
demo
├── account - Manage accounts
│   ├── fetch <ACCOUNT_ID> - Fetch an account
│   └── list (alias: ls) - List accounts
└── self - Inspect this CLI without running application logic
    ├── list (alias: ls) - Display runnable command paths as a flat list.
    └── tree - Display all visible commands as a tree.
```

List:

```text
demo account fetch <ACCOUNT_ID> - Fetch an account
demo account list (alias: ls) - List accounts
demo self list (alias: ls) - Display runnable command paths as a flat list.
demo self tree - Display all visible commands as a tree.
```

## Output behavior

- By default, children are sorted alphabetically; arguments retain clap's
  registration order. Declaration order can be selected for commands.
- Hidden commands/arguments are omitted unless `include_hidden(true)` is set.
  Only visible command aliases are shown.
- Help/version actions and clap-generated help subcommands are always omitted.
  User-defined commands named `help` are preserved. As required by clap, disable
  automatic help subcommands on their parent with `.disable_help_subcommand(true)`.
  This distinction also works with commands that have already been built.
- Long option names are preferred by default; short names can be preferred instead.
  Both modes fall back to whichever spelling is available.
- Optional arguments are bracketed; required arguments are not. Value names and
  multiple-value arguments are shown, including optional option values.
- clap builds a clone internally to resolve argument settings and inherited global
  arguments. The original command is not modified.
- Tree output includes every visible command. List output includes leaves and
  parents that do not require subcommands.
- Descriptions are normalized to one line. Tree descriptions appear on children;
  list descriptions also appear on the root if it is runnable.
- These are **discovery synopses**, not guaranteed executable shell commands.
  Argument groups, conflicts, conditional requirements, possible/default values,
  and non-global ancestor arguments are not expanded. Repeatable actions are not
  expanded into repeated occurrences. Hidden required arguments can still be
  required at runtime. clap external-subcommand placeholders are not enumerated.
- Output is colorless and ends each emitted line with a newline. An empty list
  has no trailing newline.

## Errors and panics

Writer and stdout failures are returned as `std::io::Result`, including broken
pipes. Rendering builds a clone with `clap::Command::build`: invalid command
definitions may panic according to clap's checks, particularly in debug builds.
Deferred command builders may also panic. These programmer errors are not caught
or converted into I/O errors. The original command is never modified.

## Development

Run checks for both the builder-only and derive-enabled configurations:

```sh
just lint   # Formatting and strict Clippy checks
just check  # Compilation, integration tests, and doctests
```

Both recipes are non-mutating checks; `just lint` does not automatically reformat
source files.

## Versioning

Source control keeps `0.0.0` as the package version.
[just](https://just.systems) recipes temporarily stamp
a real version into `Cargo.toml`; `cargo check` updates `Cargo.lock` to match.
Only tags starting with `v` (for example, `v0.1.0`) are considered for version
calculation; unrelated tags are ignored.

```sh
just next-version          # Next patch version from v* tags (requires doxxer)
just build                 # Build that version, then restore 0.0.0
just build 0.1.0           # Build an explicit version, then restore 0.0.0
just set-version 0.1.0     # Stamp a version before packaging/publishing
just reset-version 0.1.0   # Restore the source placeholder afterward
```

The build recipe resets the version after a successful build.
If a build fails, run `just reset-version <version>` manually.

## License

MIT. See [LICENSE](LICENSE).
