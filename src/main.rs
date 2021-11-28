extern crate serde_derive;
mod config;
mod filter_client;
use clap::{App, Arg, SubCommand};
use config::Selection;
use filter_client::FilterClient;
use flexi_logger::{self, Cleanup, Criterion, FileSpec, Naming};
use log;

fn main() {
    let matches = App::new("imap-sorter")
        .version("1.0")
        .author("Malte Müller <malte@malte-mueller.eu>")
        // .about("Does awesome things")
        .arg(
            Arg::with_name("config")
                .short("c")
                .help("config file")
                .long("config")
                .takes_value(true)
                .default_value("./config.toml"), // TODO change
        )
        .subcommand(SubCommand::with_name("list").about("list folders"))
        .subcommand(SubCommand::with_name("sort").about("sort mail"))
        .get_matches();
    let conf_path = matches.value_of("config").unwrap();
    let config = config::read_config(conf_path);
    flexi_logger::Logger::try_with_str(config.log_level)
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .suppress_timestamp()
                .directory(config.log_directory),
        )
        .append()
        .rotate(Criterion::Size(1048576), Naming::Numbers, Cleanup::Never)
        .start()
        .unwrap();

    for account in config.accounts {
        let mut client = FilterClient::new(
            &account.address,
            account.port,
            &account.username,
            &account.passowrd,
        )
        .expect("cant establish session");

        if let Some(_) = matches.subcommand_matches("list") {
            let boxes = client.list_boxes().expect("List Error!");
            for b in boxes {
                println!("{}: {}", account.address, b);
            }
        } else if let Some(_) = matches.subcommand_matches("sort") {
            for filter in account.filters {
                let mut messages = match filter.selection {
                    Selection::All() => client.get_all(&filter.source),
                    Selection::Latest(n) => client.get_n(&filter.source, n),
                    Selection::Unread() => client.get_unread(&filter.source),
                }
                .unwrap();
                let start_count = messages.len();

                messages = client
                    .filter(
                        &filter.source,
                        &messages,
                        &filter.patterns,
                        &filter.destination,
                    )
                    .unwrap();
                let count = start_count - messages.len();
                if count > 0 {
                    log::info!(
                        "{} {} -> {} ({:?}) moved {} messages.",
                        account.username,
                        filter.source,
                        filter.destination,
                        filter.patterns,
                        count
                    );
                }
            }
        }
    }
}
