#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Whatsapp,
    Discord,
    Messages,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub from_me: bool,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct Conversation {
    pub platform: Platform,
    pub user: String,
    pub messages: Vec<Message>,
    pub jid: String,
}
