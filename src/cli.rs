use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("mycochip")
        .about("Create a network of devices")
        .subcommand(Command::new("up")
            .about("Bring up a network of devices in a given configuration")
            .arg(Arg::new("config-file")
                .help("Configuration file")
                .required(false)))
        .subcommand(Command::new("list")
            .about("List running machines"))
        .subcommand(Command::new("rx")
            .about("Read characters sent by a node")
            .arg(Arg::new("node")
                .help("Node name")
                .required(true)))
}
