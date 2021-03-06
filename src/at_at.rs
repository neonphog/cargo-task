//! AtAt Encoded KV Parsing

use std::io::Read;

/// line feed
const LF: u8 = 10;

/// carriage return
const CR: u8 = 13;

/// '@' character
const AT: u8 = 64;

/// Type returned from AtAt parsing.
#[derive(Debug)]
pub enum AtAtParseItem {
    /// We parsed out a key-value item.
    KeyValue(String, String),
    /// Raw data passing through parser.
    Data(Vec<u8>),
}

/// internal parse state
enum State {
    /// middle of not @@ characters
    Waiting,

    /// at line start - look for @
    LineStart,

    /// we found an '@' at line-start - gather a name
    GatherName(Vec<u8>),

    /// we found a second '@' start gathering value
    GatherValue(Vec<u8>, Vec<u8>),

    /// we found a first termination '@' if we get another it'll be a N/V
    FirstAt(Vec<u8>, Vec<u8>),
}

/// AtAt Encoded KV Parser
pub struct AtAtParser<R: Read> {
    reader: R,
    raw_buf: [u8; 4096],
    state: Option<State>,
    eof: bool,
}

impl<R: Read> AtAtParser<R> {
    /// Wrap a reader in an AtAt parser.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            raw_buf: [0; 4096],
            state: Some(State::LineStart),
            eof: false,
        }
    }

    /// Execute one iteration of parsing.
    /// A 'None' result indicates the reader is complete (EOF).
    /// An empty Vec result may simply mean we need to wait for more data.
    pub fn parse(&mut self) -> Option<Vec<AtAtParseItem>> {
        if self.eof {
            return None;
        }

        let read = match self.reader.read(&mut self.raw_buf) {
            Ok(read) => read,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return Some(Vec::with_capacity(0))
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                return Some(Vec::with_capacity(0))
            }
            Err(e) => crate::ct_fatal!("{:?}", e),
        };

        if read == 0 {
            self.eof = true;
            self.state = None;
            return None;
        }

        let mut out = vec![AtAtParseItem::Data(self.raw_buf[..read].to_vec())];

        for c in self.raw_buf[..read].iter().cloned() {
            self.state = Some(match self.state.take().unwrap() {
                State::Waiting => {
                    if c == LF || c == CR {
                        State::LineStart
                    } else {
                        State::Waiting
                    }
                }
                State::LineStart => {
                    if c == AT {
                        State::GatherName(Vec::new())
                    } else if c == LF || c == CR {
                        State::LineStart
                    } else {
                        State::Waiting
                    }
                }
                State::GatherName(mut name) => {
                    if c == AT {
                        State::GatherValue(name, Vec::new())
                    } else {
                        name.push(c);
                        State::GatherName(name)
                    }
                }
                State::GatherValue(name, mut value) => {
                    if c == AT {
                        State::FirstAt(name, value)
                    } else {
                        value.push(c);
                        State::GatherValue(name, value)
                    }
                }
                State::FirstAt(name, mut value) => {
                    if c == AT {
                        out.push(AtAtParseItem::KeyValue(
                            String::from_utf8_lossy(&name).trim().to_string(),
                            String::from_utf8_lossy(&value).trim().to_string(),
                        ));
                        State::Waiting
                    } else {
                        value.push(64);
                        State::GatherValue(name, value)
                    }
                }
            });
        }

        Some(out)
    }
}
