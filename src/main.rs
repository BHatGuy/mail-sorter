extern crate serde_derive;
mod config;
mod filter_client;
use clap::{App, Arg, SubCommand};
use filter_client::FilterClient;

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

    for account in config.accounts {
        let mut client = FilterClient::new(
            &account.address,
            account.port,
            &account.username,
            &account.passowrd,
        )
        .expect("cant establish session");

        if let Some(_) = matches.subcommand_matches("list") {
            println!("Folders in {} ({}):", account.username, account.address);
            let boxes = client.list_boxes().expect("List Error!");
            for b in boxes {
                println!("\t{}", b);
            }
        } else if let Some(_) = matches.subcommand_matches("sort") {
            for filter in account.filters {
                let mut messages = client.get_unread(&filter.source).unwrap();
                let start_count = messages.len();
                print!(
                    "Filter {} -> {} ({:?}) ... ",
                    filter.source, filter.destination, filter.patterns
                );

                messages = client
                    .filter(
                        &filter.source,
                        &messages,
                        &filter.patterns,
                        &filter.destination,
                    )
                    .unwrap();
                println!("moved {} messages.", start_count - messages.len());
            }
        }
    }
}
