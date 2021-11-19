use crate::config::Pattern;
use imap;
use imap::types::Uid;
use native_tls;
use std::collections::HashSet;

pub struct FilterClient {
    session: imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
}

impl FilterClient {
    pub fn new(host: &str, port: u16, user: &str, password: &str) -> imap::error::Result<Self> {
        let client = imap::ClientBuilder::new(host, port).native_tls()?;

        let session = client.login(user, password).map_err(|e| e.0)?;

        Ok(Self { session })
    }

    pub fn list_boxes(&mut self) -> imap::error::Result<Vec<String>> {
        let boxes = self.session.list(Some(""), Some("*"))?;
        let res = boxes.iter().map(|b| String::from(b.name())).collect();
        Ok(res)
    }

    fn get(&mut self, seq_set: &str) -> imap::error::Result<HashSet<Uid>> {
        let mut unread = HashSet::new();
        let fetched = &self.session.fetch(seq_set, "UID")?;
        for message in fetched {
            unread.insert(message.uid.unwrap());
        }
        Ok(unread)
    }

    pub fn get_unread(&mut self, mailbox: &str) -> imap::error::Result<HashSet<Uid>> {
        self.session.select(mailbox)?;
        let unread = self.session.search("NOT SEEN")?;
        let seq_set = Vec::from_iter(unread.iter().map(|x| format!("{}", x))).join(",");
        self.get(&seq_set)
    }

    pub fn get_all(&mut self, mailbox: &str) -> imap::error::Result<HashSet<Uid>> {
        let mb = self.session.select(mailbox)?;
        let count = mb.exists;

        self.get(&format!("1:{}", count))
    }

    pub fn filter(
        &mut self,
        mailbox: &str,
        messages: &HashSet<Uid>,
        patterns: &Vec<Pattern>,
        dest: &str,
    ) -> imap::error::Result<HashSet<u32>> {
        let mut moved = HashSet::new();
        self.session.select(mailbox)?;
        let uid_set = Vec::from_iter(messages.iter().map(|x| format!("{}", x))).join(",");
        let fetched = &self.session.uid_fetch(
            uid_set,
            "(FLAGS INTERNALDATE RFC822.SIZE ENVELOPE UID BODY.PEEK[])",
        )?;
        for f in fetched {
            for p in patterns {
                if check_pattern(&p, f) {
                    self.session.uid_mv(format!("{}", f.uid.unwrap()), dest)?;
                    moved.insert(f.uid.unwrap());
                    break;
                }
            }
        }
        Ok(messages - &moved)
    }
}

// TODO refactor?
fn check_pattern(p: &Pattern, f: &imap::types::Fetch) -> bool {
    match p {
        Pattern::From(from_pattern) => {
            if let Some(envelope) = f.envelope() {
                if let Some(from) = envelope.from.as_ref() {
                    for address in from {
                        let mailbox =
                            std::str::from_utf8(&address.mailbox.as_ref().unwrap()).unwrap();
                        let host = std::str::from_utf8(&address.host.as_ref().unwrap()).unwrap();
                        let from_address = format!("{}@{}", mailbox, host);
                        if from_address.contains(from_pattern) {
                            return true;
                        }
                    }
                }
            }
        }
        Pattern::Subject(subject_pattern) => {
            if let Some(envelope) = f.envelope() {
                if let Some(subject) = envelope.subject.as_ref() {
                    let subject = std::str::from_utf8(subject).unwrap();
                    if subject.contains(subject_pattern) {
                        return true;
                    }
                }
            }
        }
        Pattern::Content(content) => {
            let body =
                dbg!(std::str::from_utf8(f.body().expect("no body?")).expect("no valid string"));
            return body.contains(content);
        }
    };
    return false;
}
