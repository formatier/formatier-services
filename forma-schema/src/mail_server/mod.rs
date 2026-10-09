use crate::inline_mod;

inline_mod!(dto);

pub struct ExternalMail {
    pub subject: Box<str>,
    pub body: Box<str>,
    pub destination: Box<str>,
    pub extensions: ExternalMailE2eExtension,
}

pub struct ExternalMailExtensions {
    pub ttl: Option<u32>,
    pub e2e: Option<ExternalMailE2eExtension>,
}

pub struct ExternalMailE2eExtension {}
