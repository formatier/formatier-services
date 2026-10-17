pub enum DataConverterType {
    Json,
    Postcard,
    Cobr,
    Yaml,
    MessagePack,
    Toml,
    Bson,
    Csv,
}

pub enum DataConverterMode {
    Default,
    NoWrapperWithSending,
    NoWrapperWithoutSending,
}

pub struct DataConverterRequest {
    pub data: Box<str>,
    pub from: DataConverterType,
    pub to: DataConverterType,
    pub mode: DataConverterMode,
}

pub struct DataConverterReply {
    pub data: Box<str>,
}
