use serde::Serialize;

pub(crate) struct JSONParser {}

pub(crate) type JSONError = serde_json::Error;

impl JSONParser {
    #[inline]
    pub fn from_str<'a,
        T>(s: &'a str) -> Result<T, JSONError>
        where
            T: serde::de::Deserialize<'a>,
    {
        serde_json::from_str(s)
    }
    #[inline]
    pub fn to_string<T>(value: &T) -> Result<String,JSONError>
        where
            T: ?Sized + Serialize,
    {
        serde_json::to_string(value)
    }
}

