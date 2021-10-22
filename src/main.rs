mod filter_client;
use filter_client::FilterClient;

fn main() {
    let mut client = FilterClient::new("imap.gmx.net", 993, "malte-m@gmx.net", "thomasB661")
        .expect("cant establish session");

    let unread = client.get_all("INBOX").unwrap();


    let rest = client.filter("INBOX", &unread, "no-reply@mail.instagram.com", "Test").unwrap();
    println!("{:?}", unread.len() - rest.len());

}
