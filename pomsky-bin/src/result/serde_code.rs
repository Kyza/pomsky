use pomsky::diagnose::DiagnosticCode;
use serde::{
    de::{Error, Expected, Unexpected, Visitor},
    Deserializer, Serializer,
};

pub(super) fn serialize<S>(value: &Option<DiagnosticCode>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.collect_str(value),
        None => serializer.serialize_none(),
    }
}

pub(super) fn deserialize<'de, D>(d: D) -> Result<Option<DiagnosticCode>, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(CodeVisitor).map(Some)
}

struct CodeVisitor;

impl<'de> Visitor<'de> for CodeVisitor {
    type Value = DiagnosticCode;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "an integer that is a valid diagnostic code")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.trim_start_matches("P")
            .parse::<u16>()
            .map_or_else(|_| Err(()), DiagnosticCode::try_from)
            .map_err(|_| Error::invalid_value(Unexpected::Str(v.into()), &ExpectedCode))
    }
}

struct ExpectedCode;

impl Expected for ExpectedCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "diagnostic code")
    }
}
