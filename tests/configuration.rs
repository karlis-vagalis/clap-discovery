use clap::{Arg, ArgAction, Command};
use clap_discovery::{Discovery, OptionStyle, Renderer, SortOrder, TreeStyle, list, tree};

#[test]
fn defaults_match_convenience_functions() {
    let command = Command::new("app")
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .subcommand(Command::new("zebra").about("Last"))
        .subcommand(Command::new("alpha").visible_alias("a"));
    let renderer = Renderer::default();
    assert_eq!(renderer, Renderer::new());
    assert_eq!(renderer.tree(&command), tree(&command));
    assert_eq!(renderer.list(&command), list(&command));
    for format in [Discovery::Tree, Discovery::List] {
        assert_eq!(renderer.render(&command, format), format.render(&command));
    }
}

#[test]
fn ascii_connectors_preserve_nested_branches() {
    let command = Command::new("app")
        .subcommand(Command::new("alpha").subcommand(Command::new("child")))
        .subcommand(Command::new("zebra").subcommand(Command::new("child")));
    assert_eq!(
        Renderer::new().tree_style(TreeStyle::Ascii).tree(&command),
        "app\n|-- alpha\n|   `-- child\n`-- zebra\n    `-- child\n"
    );
    assert_eq!(
        Renderer::new().tree_style(TreeStyle::Ascii).list(&command),
        list(&command)
    );
}

#[test]
fn declaration_order_applies_to_tree_and_list_at_every_depth() {
    let command = Command::new("app")
        .subcommand_required(true)
        .subcommand(
            Command::new("zebra")
                .subcommand_required(true)
                .subcommand(Command::new("two"))
                .subcommand(Command::new("one")),
        )
        .subcommand(Command::new("alpha"));
    let renderer = Renderer::new().sort_order(SortOrder::Declaration);
    assert_eq!(
        renderer.tree(&command),
        "app\n├── zebra\n│   ├── two\n│   └── one\n└── alpha\n"
    );
    assert_eq!(
        renderer.list(&command),
        "app zebra two\napp zebra one\napp alpha\n"
    );
}

#[test]
fn include_hidden_shows_commands_and_arguments_but_not_hidden_aliases() {
    let command = Command::new("app")
        .arg(
            Arg::new("secret")
                .long("secret")
                .hide(true)
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("internal")
                .hide(true)
                .alias("private")
                .visible_alias("i"),
        );
    let renderer = Renderer::new().include_hidden(true);
    assert_eq!(tree(&command), "app\n");
    assert_eq!(
        renderer.tree(&command),
        "app [--secret]\n└── internal (alias: i)\n"
    );
    assert_eq!(
        renderer.list(&command),
        "app [--secret]\napp internal (alias: i)\n"
    );
}

#[test]
fn summary_components_can_be_disabled_independently() {
    let command = Command::new("app").about("Root").subcommand(
        Command::new("status")
            .visible_alias("s")
            .about("Check status")
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .action(ArgAction::SetTrue),
            ),
    );
    assert_eq!(
        Renderer::new().show_descriptions(false).list(&command),
        "app\napp status (alias: s) [--verbose]\n"
    );
    assert_eq!(
        Renderer::new().show_aliases(false).list(&command),
        "app - Root\napp status [--verbose] - Check status\n"
    );
    assert_eq!(
        Renderer::new().show_arguments(false).list(&command),
        "app - Root\napp status (alias: s) - Check status\n"
    );
    assert_eq!(
        Renderer::new()
            .show_arguments(false)
            .show_aliases(false)
            .show_descriptions(false)
            .tree(&command),
        "app\n└── status\n"
    );
}

#[test]
fn preferred_option_style_falls_back_to_available_spelling() {
    let command = Command::new("app")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE"),
        )
        .arg(Arg::new("quiet").short('q').action(ArgAction::SetTrue))
        .arg(Arg::new("force").long("force").action(ArgAction::SetTrue));
    assert_eq!(tree(&command), "app [--output <FILE>] [-q] [--force]\n");
    assert_eq!(
        Renderer::new()
            .option_style(OptionStyle::Short)
            .tree(&command),
        "app [-o <FILE>] [-q] [--force]\n"
    );
}

#[test]
fn customized_writes_use_renderer_settings_and_are_reusable() {
    let renderer = Renderer::new()
        .tree_style(TreeStyle::Ascii)
        .show_descriptions(false);
    let command = Command::new("app").subcommand(Command::new("child").about("Child"));
    for format in [Discovery::Tree, Discovery::List] {
        let mut output = Vec::new();
        renderer.write(&command, format, &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            renderer.render(&command, format)
        );
    }
    assert_eq!(renderer.tree(&command), "app\n`-- child\n");
    assert_eq!(tree(&command), "app\n└── child - Child\n");
}

#[test]
fn generated_help_is_omitted_even_for_prebuilt_commands_and_hidden_inclusion() {
    let mut command =
        Command::new("app").subcommand(Command::new("account").subcommand(Command::new("list")));
    command.build();
    assert!(
        command
            .get_subcommands()
            .any(|child| child.get_name() == "help")
    );
    let expected = "app\n└── account\n    └── list\n";
    assert_eq!(tree(&command), expected);
    assert_eq!(
        Renderer::new().include_hidden(true).tree(&command),
        expected
    );
    assert_eq!(list(&command), "app\napp account\napp account list\n");
}

#[test]
fn custom_help_commands_are_preserved_before_and_after_building() {
    let mut command = Command::new("app")
        .disable_help_subcommand(true)
        .subcommand(Command::new("help").about("Contact support"))
        .subcommand(Command::new("status"));
    for _ in 0..2 {
        assert_eq!(
            tree(&command),
            "app\n├── help - Contact support\n└── status\n"
        );
        assert_eq!(
            list(&command),
            "app\napp help - Contact support\napp status\n"
        );
        command.build();
    }
}

#[test]
fn nested_custom_help_uses_its_parents_setting() {
    let command = Command::new("app").subcommand(
        Command::new("tools")
            .disable_help_subcommand(true)
            .subcommand(Command::new("help").subcommand(Command::new("topic"))),
    );
    assert_eq!(
        tree(&command),
        "app\n└── tools\n    └── help\n        └── topic\n"
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Argument names must be unique")]
fn invalid_clap_definitions_preserve_claps_panic_behavior() {
    let command = Command::new("app")
        .arg(Arg::new("duplicate").long("first"))
        .arg(Arg::new("duplicate").long("second"));
    tree(&command);
}
