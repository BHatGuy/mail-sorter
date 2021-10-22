mod imap_cli;
use imap_cli::ImapCli;

fn main() {
    let mut client = ImapCli::new("imap.gmx.net", 993, "malte-m@gmx.net", "thomasB661")
        .expect("cant establish session");
    let boxes = client.list_boxes().unwrap();

    println!("{:?}", boxes);
}
