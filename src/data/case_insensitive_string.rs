use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct CaseInsensitiveString(String);

impl From<&str> for CaseInsensitiveString {
    fn from(s: &str) -> Self {
        Self(s.to_lowercase())
    }
}

impl<'a> bonsaidb::core::key::Key<'a> for CaseInsensitiveString {
    fn from_ord_bytes(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        std::str::from_utf8(bytes).map(|x| x.into())
    }
}

impl<'a> bonsaidb::core::key::KeyEncoding<'a, Self> for CaseInsensitiveString {
    type Error = std::str::Utf8Error;

    const LENGTH: Option<usize> = None;

    fn as_ord_bytes(&'a self) -> Result<Cow<'a, [u8]>, Self::Error> {
        Ok(Cow::Borrowed(self.0.as_bytes()))
    }
}

impl<'a> bonsaidb::core::key::KeyEncoding<'a, CaseInsensitiveString> for &str {
    type Error = std::str::Utf8Error;

    const LENGTH: Option<usize> = None;

    fn as_ord_bytes(&'a self) -> Result<Cow<'a, [u8]>, Self::Error> {
        Ok(Cow::Owned(self.to_lowercase().into_bytes()))
    }
}

impl<'a> bonsaidb::core::key::KeyEncoding<'a, CaseInsensitiveString> for &String {
    type Error = std::str::Utf8Error;

    const LENGTH: Option<usize> = None;

    fn as_ord_bytes(&'a self) -> Result<Cow<'a, [u8]>, Self::Error> {
        Ok(Cow::Owned(self.to_lowercase().into_bytes()))
    }
}
