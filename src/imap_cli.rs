use imap;
use native_tls;

pub struct ImapCli {
    session: imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
}

impl ImapCli {
    pub fn new<S: AsRef<str>>(
        host: S,
        port: u16,
        user: S,
        password: S,
    ) -> imap::error::Result<Self> {
        let client = imap::ClientBuilder::new(host, port).native_tls()?;

        let session = client.login(user, password).map_err(|e| e.0)?;

        Ok(Self { session })
    }

    pub fn list_boxes(&mut self) -> imap::error::Result<Vec<String>> {
        let boxes = self.session.list(Some(""), Some("*"))?;
        let res = boxes.iter().map(|b| String::from(b.name())).collect();
        Ok(res)
    }
}
