use clap::{AppSettings, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(setting(AppSettings::NoBinaryName))]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
#[clap(global_setting(AppSettings::DisableVersionFlag))]
pub struct Command {
    #[clap(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    #[clap(subcommand)]
    Print(PrintCmd),
}

#[derive(Debug, Subcommand)]
#[clap(about = "Print resources")]
#[clap(visible_aliases = &["get", "show", "p"])]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
enum PrintCmd {
    Requests(PrintRequestsCmd),
    Variables(PrintVariablesCmd),
    Environments(PrintEnvironmentsCmd),
    Workspaces(PrintWorkspacesCmd),
}

#[derive(Debug, Parser)]
#[clap(about = "Print requests")]
#[clap(visible_aliases = &["request", "reqs", "req", "r"])]
struct PrintRequestsCmd {}

#[derive(Debug, Parser)]
#[clap(about = "Print variables")]
#[clap(visible_aliases = &["variable", "vars", "var", "v"])]
struct PrintVariablesCmd {}

#[derive(Debug, Parser)]
#[clap(about = "Print environments")]
#[clap(visible_aliases = &["environment", "envs", "env", "e"])]
struct PrintEnvironmentsCmd {}

#[derive(Debug, Parser)]
#[clap(about = "Print workspaces")]
#[clap(visible_aliases = &["workspace", "ws", "w"])]
struct PrintWorkspacesCmd {}
