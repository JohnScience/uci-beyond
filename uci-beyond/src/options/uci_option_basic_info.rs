use crate::options::{UciOptionKind, UciOptionType};

pub enum UciOptionBasicInfo<S> {
    Standard(UciOptionKind),
    Custom { name: S, r#type: UciOptionType },
}

impl<'a> UciOptionBasicInfo<&'a str> {
    pub fn name(self) -> &'a str {
        match self {
            UciOptionBasicInfo::Standard(kind) => kind.name(),
            UciOptionBasicInfo::Custom { name, .. } => name,
        }
    }

    pub fn r#type(self) -> UciOptionType {
        match self {
            UciOptionBasicInfo::Standard(kind) => kind.r#type(),
            UciOptionBasicInfo::Custom { r#type, .. } => r#type,
        }
    }
}
