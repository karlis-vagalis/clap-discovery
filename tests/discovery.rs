use clap::{Arg, ArgAction, Command};
use clap_discovery::{Discovery, list, tree};
use std::io;

fn fixture() -> Command {
    Command::new("app")
        .version("1.0")
        .subcommand_required(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count)
                .global(true),
        )
        .subcommand(Command::new("zebra").about("Last command"))
        .subcommand(Command::new("internal").hide(true))
        .subcommand(
            Command::new("account")
                .about("Manage accounts")
                .subcommand_required(true)
                .subcommand(
                    Command::new("list")
                        .visible_alias("ls")
                        .alias("secret")
                        .about("List accounts"),
                )
                .subcommand(
                    Command::new("fetch")
                        .about("Fetch an account")
                        .arg(Arg::new("id").required(true).value_name("ID")),
                ),
        )
}

#[test]
fn tree_snapshot() {
    assert_eq!(
        tree(&fixture()),
        "app [--verbose]\n├── account [--verbose] - Manage accounts\n│   ├── fetch <ID> [--verbose] - Fetch an account\n│   └── list (alias: ls) [--verbose] - List accounts\n└── zebra [--verbose] - Last command\n"
    );
}

#[test]
fn list_snapshot() {
    assert_eq!(
        list(&fixture()),
        "app account fetch <ID> [--verbose] - Fetch an account\napp account list (alias: ls) [--verbose] - List accounts\napp zebra [--verbose] - Last command\n"
    );
}

#[test]
fn includes_parents_with_optional_subcommands() {
    let command = Command::new("app")
        .about("Root")
        .subcommand(Command::new("child"));
    assert_eq!(list(&command), "app - Root\napp child\n");
    assert_eq!(tree(&command), "app\n└── child\n");
}

#[test]
fn empty_command_and_original_are_unchanged() {
    let command = Command::new("app");
    assert_eq!(command.get_arguments().count(), 0);
    assert_eq!(tree(&command), "app\n");
    assert_eq!(list(&command), "app\n");
    assert_eq!(command.get_arguments().count(), 0);
    assert_eq!(command.get_subcommands().count(), 0);
}

#[test]
fn hidden_children_do_not_make_required_parent_runnable() {
    let command = Command::new("app")
        .subcommand_required(true)
        .subcommand(Command::new("secret").hide(true));
    assert_eq!(list(&command), "");
    assert_eq!(tree(&command), "app\n");
}

#[test]
fn formats_argument_shapes_and_omits_hidden_and_help_actions() {
    let command = Command::new("app")
        .disable_help_flag(true)
        .arg(
            Arg::new("help")
                .long("custom-help")
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("secret").long("secret").hide(true))
        .arg(Arg::new("quiet").short('q').action(ArgAction::SetTrue))
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::new("pair")
                .long("pair")
                .value_names(["KEY", "VALUE"])
                .num_args(2),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .num_args(0..=1)
                .default_missing_value("auto"),
        )
        .arg(Arg::new("files").num_args(1..).value_name("FILE"));
    assert_eq!(
        tree(&command),
        "app [--custom-help] [-q] --output <FILE> [--pair <KEY> <VALUE>] [--color [<color>]] [<FILE>...]\n"
    );
}

#[test]
fn excludes_all_help_and_version_actions_even_with_custom_ids() {
    let command = Command::new("app")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(Arg::new("assist").long("assist").action(ArgAction::Help))
        .arg(Arg::new("brief").long("brief").action(ArgAction::HelpShort))
        .arg(Arg::new("full").long("full").action(ArgAction::HelpLong))
        .arg(
            Arg::new("release")
                .long("release")
                .action(ArgAction::Version),
        )
        .version("1.0");
    assert_eq!(tree(&command), "app\n");
}

#[test]
fn multiline_descriptions_stay_on_one_line_and_long_about_is_a_fallback() {
    let command = Command::new("app")
        .subcommand(Command::new("child").long_about("  First line\n Second\tline  "));
    assert_eq!(list(&command), "app\napp child - First line Second line\n");
}

#[test]
fn writer_matches_render_and_propagates_errors() {
    for format in [Discovery::Tree, Discovery::List] {
        let mut buffer = Vec::new();
        format.write(&fixture(), &mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            format.render(&fixture())
        );
    }

    struct Broken;
    impl io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    assert_eq!(
        Discovery::Tree
            .write(&fixture(), Broken)
            .unwrap_err()
            .kind(),
        io::ErrorKind::BrokenPipe
    );
}

#[cfg(feature = "derive")]
mod derive_integration {
    use super::*;
    use clap::{CommandFactory, Parser, Subcommand};

    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(Subcommand)]
    enum Commands {
        #[command(name = "self")]
        Discover {
            #[command(subcommand)]
            format: Discovery,
        },
    }

    #[test]
    fn supports_tree_list_and_ls() {
        Cli::command().debug_assert();
        for (name, expected) in [
            ("tree", Discovery::Tree),
            ("list", Discovery::List),
            ("ls", Discovery::List),
        ] {
            let cli = Cli::try_parse_from(["app", "self", name]).unwrap();
            let Commands::Discover { format } = cli.command;
            assert_eq!(format, expected);
        }
    }
}
