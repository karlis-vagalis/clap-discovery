use crate::Discovery;
use clap::{Arg, ArgAction, Command};
use std::fmt::Write as _;
use std::io::{self, Write};

/// Ordering of sibling commands. Argument order is unaffected.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort siblings alphabetically by canonical command name.
    #[default]
    Alphabetical,
    /// Preserve clap's command registration order.
    Declaration,
}

/// Connector characters used in tree output.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TreeStyle {
    /// Unicode box-drawing characters.
    #[default]
    Unicode,
    /// ASCII connectors, suitable for basic terminals.
    Ascii,
}

impl TreeStyle {
    fn connector(self, last: bool) -> &'static str {
        match (self, last) {
            (Self::Unicode, false) => "├── ",
            (Self::Unicode, true) => "└── ",
            (Self::Ascii, false) => "|-- ",
            (Self::Ascii, true) => "`-- ",
        }
    }

    fn continuation(self) -> &'static str {
        match self {
            Self::Unicode => "│   ",
            Self::Ascii => "|   ",
        }
    }
}

/// Preferred spelling when an option has both long and short names.
///
/// Falls back to the other spelling when the preferred one is unavailable.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OptionStyle {
    /// Prefer `--long`.
    #[default]
    Long,
    /// Prefer `-s`.
    Short,
}

/// Reusable discovery renderer with consuming builder methods.
///
/// Defaults match [`crate::tree`] and [`crate::list`]: visible items only,
/// alphabetically sorted commands, Unicode connectors, long options preferred,
/// and arguments, aliases, and descriptions shown.
///
/// ```
/// use clap::Command;
/// use clap_discovery::{Renderer, SortOrder, TreeStyle};
///
/// let renderer = Renderer::new()
///     .tree_style(TreeStyle::Ascii)
///     .sort_order(SortOrder::Declaration)
///     .show_descriptions(false);
/// let command = Command::new("app")
///     .subcommand(Command::new("status").about("Check status"));
/// assert_eq!(renderer.tree(&command), "app\n\x60-- status\n");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Renderer {
    include_hidden: bool,
    show_descriptions: bool,
    show_aliases: bool,
    show_arguments: bool,
    sort_order: SortOrder,
    tree_style: TreeStyle,
    option_style: OptionStyle,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            include_hidden: false,
            show_descriptions: true,
            show_aliases: true,
            show_arguments: true,
            sort_order: SortOrder::default(),
            tree_style: TreeStyle::default(),
            option_style: OptionStyle::default(),
        }
    }
}

impl Renderer {
    /// Create a renderer with sensible default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Include hidden commands and arguments (default: false).
    ///
    /// Hidden aliases and help/version actions are still omitted.
    pub fn include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }

    /// Show one-line descriptions (default: true).
    ///
    /// Tree output describes children, not the root. List output describes
    /// every emitted command, including the root when runnable.
    pub fn show_descriptions(mut self, show: bool) -> Self {
        self.show_descriptions = show;
        self
    }

    /// Show visible command aliases (default: true).
    ///
    /// Tree output shows aliases on children, not the root.
    pub fn show_aliases(mut self, show: bool) -> Self {
        self.show_aliases = show;
        self
    }

    /// Show argument synopses (default: true).
    pub fn show_arguments(mut self, show: bool) -> Self {
        self.show_arguments = show;
        self
    }

    /// Choose sibling ordering (default: alphabetical).
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }

    /// Choose tree connectors (default: Unicode). Does not affect lists.
    pub fn tree_style(mut self, style: TreeStyle) -> Self {
        self.tree_style = style;
        self
    }

    /// Choose preferred option spelling (default: long).
    pub fn option_style(mut self, style: OptionStyle) -> Self {
        self.option_style = style;
        self
    }

    /// Render the complete command tree, ending each emitted line with a newline.
    ///
    /// Help/version actions and clap's generated help subcommands are omitted.
    /// User-defined `help` subcommands are preserved; as required by clap, their
    /// parent must disable automatic help subcommands.
    ///
    /// A clone is built internally to resolve inferred argument settings and
    /// inherited global arguments without modifying the original command.
    ///
    /// # Panics
    ///
    /// May panic when [`Command::build`] encounters an invalid definition
    /// (particularly in debug builds), or a deferred builder panics.
    pub fn tree(&self, command: &Command) -> String {
        let prepared = PreparedCommand::new(command);
        let command = &prepared.0;
        let mut output = String::new();
        writeln!(output, "{}{}", command.get_name(), self.arguments(command)).unwrap();
        self.tree_children(command, "", &mut output);
        output
    }

    /// Render runnable command paths, ending each emitted line with a newline.
    ///
    /// Includes leaves and parents that do not require subcommands, in depth-first
    /// order. An empty list produces an empty string. Filtering hidden children
    /// does not make a required-subcommand parent directly runnable.
    ///
    /// Each line shows the full command path and any configured aliases,
    /// arguments, and description. These are discovery synopses, not executable
    /// shell commands: groups, conflicts, conditional requirements, and
    /// non-global ancestor arguments are not expanded.
    ///
    /// Builds a clone and applies the same help filtering as [`Self::tree`].
    ///
    /// # Panics
    ///
    /// May panic when [`Command::build`] encounters an invalid definition
    /// (particularly in debug builds), or a deferred builder panics.
    pub fn list(&self, command: &Command) -> String {
        let prepared = PreparedCommand::new(command);
        let mut output = String::new();
        self.list_commands(&prepared.0, "", &mut output);
        output
    }

    /// Render the selected format with this renderer's settings.
    ///
    /// # Panics
    ///
    /// May panic when clap builds an invalid command definition.
    pub fn render(&self, command: &Command, format: Discovery) -> String {
        match format {
            Discovery::Tree => self.tree(command),
            Discovery::List => self.list(command),
        }
    }

    /// Write the selected format to a file, buffer, or other output.
    ///
    /// # Errors
    ///
    /// Returns any error reported by the writer.
    ///
    /// # Panics
    ///
    /// May panic when clap builds an invalid command definition.
    pub fn write(
        &self,
        command: &Command,
        format: Discovery,
        mut writer: impl Write,
    ) -> io::Result<()> {
        writer.write_all(self.render(command, format).as_bytes())
    }

    /// Print the selected format to locked standard output.
    ///
    /// # Errors
    ///
    /// Returns any stdout error, including a broken pipe.
    ///
    /// # Panics
    ///
    /// May panic when clap builds an invalid command definition.
    pub fn print(&self, command: &Command, format: Discovery) -> io::Result<()> {
        self.write(command, format, io::stdout().lock())
    }

    fn children<'a>(&self, command: &'a Command) -> Vec<&'a Command> {
        let mut children: Vec<_> = command
            .get_subcommands()
            .filter(|child| {
                (self.include_hidden || !child.is_hide_set())
                    // clap reserves this name when automatic help is enabled.
                    // Inspect the parent setting, not just the child's name:
                    // custom help commands disable generation on their parent.
                    && !(child.get_name() == "help"
                        && !command.is_disable_help_subcommand_set())
            })
            .collect();
        if self.sort_order == SortOrder::Alphabetical {
            children.sort_by(|a, b| a.get_name().cmp(b.get_name()));
        }
        children
    }

    fn tree_children(&self, command: &Command, prefix: &str, output: &mut String) {
        let children = self.children(command);
        for (index, child) in children.iter().enumerate() {
            let last = index + 1 == children.len();
            let connector = self.tree_style.connector(last);
            writeln!(
                output,
                "{prefix}{connector}{}",
                self.summary(child, child.get_name())
            )
            .unwrap();
            let continuation = if last {
                "    "
            } else {
                self.tree_style.continuation()
            };
            let next_prefix = format!("{prefix}{continuation}");
            self.tree_children(child, &next_prefix, output);
        }
    }

    fn list_commands(&self, command: &Command, parent: &str, output: &mut String) {
        let path = if parent.is_empty() {
            command.get_name().to_owned()
        } else {
            format!("{parent} {}", command.get_name())
        };
        if !command.has_subcommands() || !command.is_subcommand_required_set() {
            writeln!(output, "{}", self.summary(command, &path)).unwrap();
        }
        for child in self.children(command) {
            self.list_commands(child, &path, output);
        }
    }

    fn summary(&self, command: &Command, name: &str) -> String {
        let mut result = name.to_owned();
        if self.show_aliases {
            let aliases: Vec<_> = command.get_visible_aliases().collect();
            if !aliases.is_empty() {
                write!(result, " (alias: {})", aliases.join(", ")).unwrap();
            }
        }
        result.push_str(&self.arguments(command));
        if self.show_descriptions
            && let Some(about) = command.get_about().or_else(|| command.get_long_about())
        {
            let about = about.to_string();
            let about = about.split_whitespace().collect::<Vec<_>>().join(" ");
            if !about.is_empty() {
                write!(result, " - {about}").unwrap();
            }
        }
        result
    }

    fn arguments(&self, command: &Command) -> String {
        let mut result = String::new();
        if !self.show_arguments {
            return result;
        }
        for arg in command.get_arguments().filter(|arg| {
            (self.include_hidden || !arg.is_hide_set())
                && !matches!(
                    arg.get_action(),
                    ArgAction::Help
                        | ArgAction::HelpShort
                        | ArgAction::HelpLong
                        | ArgAction::Version
                )
        }) {
            let mut synopsis = self.argument(arg);
            if !arg.is_required_set() {
                synopsis = format!("[{synopsis}]");
            }
            write!(result, " {synopsis}").unwrap();
        }
        result
    }

    fn argument(&self, arg: &Arg) -> String {
        let long = || arg.get_long().map(|long| format!("--{long}"));
        let short = || arg.get_short().map(|short| format!("-{short}"));
        let flag = match self.option_style {
            OptionStyle::Long => long().or_else(short),
            OptionStyle::Short => short().or_else(long),
        };
        if !arg.get_action().takes_values() {
            return flag.unwrap_or_else(|| arg.get_id().to_string());
        }

        let names = arg.get_value_names();
        let mut values = match names.filter(|names| !names.is_empty()) {
            Some(names) => names
                .iter()
                .map(|name| format!("<{name}>"))
                .collect::<Vec<_>>()
                .join(" "),
            None => format!("<{}>", arg.get_id()),
        };
        let range = arg.get_num_args();
        let named_count = names.map_or(1, |names| names.len());
        if range.is_some_and(|range| range.max_values() > named_count) {
            values.push_str("...");
        }
        if range.is_some_and(|range| range.min_values() == 0) {
            values = format!("[{values}]");
        }
        match flag {
            Some(flag) => format!("{flag} {values}"),
            None => values,
        }
    }
}

// Only this constructor builds command definitions. Rendering starts with this
// prepared value, keeping clap's inferred settings available at the boundary.
struct PreparedCommand(Command);

impl PreparedCommand {
    fn new(command: &Command) -> Self {
        let mut command = command.clone();
        command.build();
        Self(command)
    }
}
