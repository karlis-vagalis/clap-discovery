//! Configurable tree and flat-list self-discovery for clap applications.
//!
//! ```
//! use clap::Command;
//! use clap_discovery::{Renderer, TreeStyle};
//!
//! let command = Command::new("app").subcommand(Command::new("status"));
//! assert_eq!(clap_discovery::tree(&command), "app\n└── status\n");
//! assert_eq!(
//!     Renderer::new().tree_style(TreeStyle::Ascii).tree(&command),
//!     "app\n\x60-- status\n",
//! );
//! ```
//!
//! No proc macros are required by default. Enable the `derive` feature to use
//! [`Discovery`] as a `clap::Subcommand` in a derive-based CLI.
//!
//! # Panics
//!
//! Rendering builds a clone with [`clap::Command::build`]. Invalid command
//! definitions can panic according to clap's checks, particularly in debug
//! builds. These are programmer errors, not recoverable I/O errors. Deferred
//! command builders may also panic. The original command is never modified.

mod renderer;

pub use renderer::{OptionStyle, Renderer, SortOrder, TreeStyle};

use clap::Command;
use std::io::{self, Write};

/// Select a discovery format.
///
/// With the opt-in `derive` feature, this also implements `clap::Subcommand`
/// for embedding in your CLI. The list action has a visible `ls` alias.
/// Without that feature, the same enum and rendering methods remain available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "derive", derive(clap::Subcommand))]
pub enum Discovery {
    /// Display all visible commands as a tree.
    Tree,
    /// Display runnable command paths as a flat list.
    #[cfg_attr(feature = "derive", command(visible_alias = "ls"))]
    List,
}

impl Discovery {
    /// Render using the default [`Renderer`].
    ///
    /// # Panics
    ///
    /// May panic when clap builds an invalid command definition.
    pub fn render(self, command: &Command) -> String {
        Renderer::new().render(command, self)
    }

    /// Write using the default [`Renderer`].
    ///
    /// # Errors
    ///
    /// Returns any error reported by the writer.
    ///
    /// # Panics
    ///
    /// May panic when clap builds an invalid command definition.
    pub fn write(self, command: &Command, writer: impl Write) -> io::Result<()> {
        Renderer::new().write(command, self, writer)
    }

    /// Print using the default [`Renderer`] to locked standard output.
    ///
    /// # Errors
    ///
    /// Returns any stdout error, including a broken pipe.
    ///
    /// # Panics
    ///
    /// May panic when clap builds an invalid command definition.
    pub fn print(self, command: &Command) -> io::Result<()> {
        Renderer::new().print(command, self)
    }
}

/// Render the complete visible command tree with default settings.
///
/// See [`Renderer::tree`] for output behavior and configuration.
///
/// # Panics
///
/// May panic when clap builds an invalid command definition.
pub fn tree(command: &Command) -> String {
    Renderer::new().tree(command)
}

/// Render visible runnable command paths with default settings.
///
/// See [`Renderer::list`] for output behavior and configuration.
///
/// # Panics
///
/// May panic when clap builds an invalid command definition.
pub fn list(command: &Command) -> String {
    Renderer::new().list(command)
}
