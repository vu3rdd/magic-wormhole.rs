use serde_json;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Nameplate {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WelcomeMsg {
    motd: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Phase {
    PakePhase,
    VersionPhase,
    ApplicationPhase(i32)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum Message {
    Bind {
        appid: String,
        side: String,
    },
    Welcome {
        server_tx: f64,
        welcome:WelcomeMsg,
    },
    List {},
    Nameplates {
        nameplates: Vec<Nameplate>,
    },
    Allocate {},
    Allocated {
        nameplate: String,
    },
    Claim {
        nameplate: String,
    },
    Claimed {
        mailbox: String,
    },
    Release {
        nameplate: String,
    }, // TODO: nominally optional
    Released {},
    Open {
        mailbox: String,
    },
    Add {
        phase: Phase,
        body: String,
    },
    Message {
        side: String,
        phase: Phase,
        body: String,
        id: String,
    },
    Close {
        mailbox: String,
        mood: String,
    },
    Closed {},
    Ack {},
    Ping {
        ping: u32,
    },
    Pong {
        pong: u32,
    },
    //Error { error: String, orig: Message },
}

// Client only sends: bind, list, allocate, claim, release, open, add, close,
// ping

pub fn bind(appid: &str, side: &str) -> Message {
    Message::Bind {
        appid: appid.to_string(),
        side: side.to_string(),
    }
}
pub fn list() -> Message {
    Message::List {}
}
pub fn allocate() -> Message {
    Message::Allocate {}
}
pub fn claim(nameplate: &str) -> Message {
    Message::Claim {
        nameplate: nameplate.to_string(),
    }
}
pub fn release(nameplate: &str) -> Message {
    Message::Release {
        nameplate: nameplate.to_string(),
    }
}
pub fn open(mailbox: &str) -> Message {
    Message::Open {
        mailbox: mailbox.to_string(),
    }
}

fn phase_name(phase: Phase) -> String {
    match phase {
        Phase::PakePhase => "pake".to_string(),
        Phase::VersionPhase => "version".to_string(),
        Phase::ApplicationPhase(n) => n.to_string()
    }
}

pub fn add(phase: Phase, body: &str) -> Message {
    // TODO: make this take Vec<u8>, do the hex-encoding internally
    Message::Add {
        phase: phase,
        body: body.to_string(),
    }
}

pub fn close(mailbox: &str, mood: &str) -> Message {
    Message::Close {
        mailbox: mailbox.to_string(),
        mood: mood.to_string(),
    }
}

pub fn ping(ping: u32) -> Message {
    Message::Ping { ping: ping }
}

pub fn welcome(motd: &str, timestamp: f64) -> Message {
    Message::Welcome {
        welcome: WelcomeMsg { motd: motd.to_string() },
        server_tx: timestamp
    }
}

// Server sends: welcome, nameplates, allocated, claimed, released, message,
// closed, ack, pong, error

pub fn deserialize(s: &str) -> Message {
    serde_json::from_str(&s).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bind() {
        let m1 = bind("appid", "side1");
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_list() {
        let m1 = list();
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_allocate() {
        let m1 = allocate();
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_claim() {
        let m1 = claim("nameplate1");
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_release() {
        let m1 = release("nameplate1");
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_open() {
        let m1 = open("mailbox1");
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_add() {
        let m1 = add(Phase::VersionPhase, "body");
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_close() {
        let m1 = close("mailbox1", "mood");
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_ping() {
        let m1 = ping(123);
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_welcome() {
        let m1 = welcome("hi", 1234.56);
        let s = serde_json::to_string(&m1).unwrap();
        let m2 = deserialize(&s);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_ack() {
        let s = r#"{"type": "ack", "id": null, "server_tx": 1234.56}"#;
        let m = deserialize(&s);
        match m {
            Message::Ack {} => (),
            _ => panic!(),
        }
    }

}
