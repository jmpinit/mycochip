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
            .about("Read characters from a specific channel")
            .arg(Arg::new("channel")
                .help("Channel name")
                .required(true)))
}
