use std::borrow::Cow;
use std::slice;
use std::str::from_utf8_unchecked;
use {
    serde::*,
    std::{
        error::Error,
        fmt::{Debug, Display, Formatter, Write},
        hash::Hash,
    },
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetLocation {
    namespace: Cow<'static, str>,
    path: Cow<'static, str>,
}

impl Debug for AssetLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistrationId")
            .field("namespace", &self.namespace)
            .field("path", &self.path)
            .finish()
    }
}

impl<'de> Deserialize<'de> for AssetLocation {
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AssetLocationVisitor)
    }
}

impl Serialize for AssetLocation {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'a> TryFrom<&'a str> for AssetLocation {
    type Error = AssetLocationError<'a>;

    #[inline(always)]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::try_parse(value)
    }
}

impl Display for AssetLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_char(':')?;
        f.write_str(&self.path)
    }
}

impl Default for AssetLocation {
    #[inline(always)]
    fn default() -> Self {
        Self {
            namespace: Cow::Borrowed(Self::DEFAULT_NAMESPACE),
            path: Cow::Borrowed("air"),
        }
    }
}

impl AssetLocation {
    pub const DEFAULT_NAMESPACE: &'static str = "bevycraft";

    const SEPARATOR: u8 = b':';

    #[inline]
    pub fn try_parse(location: &str) -> Result<Self, AssetLocationError<'_>> {
        match location.split_once(':') {
            None => Self::try_with_default_namespace(location),
            Some((n, p)) => {
                if p.contains(':') {
                    return Err(AssetLocationError::IllegalFormat);
                }

                Self::try_new(n, p)
            }
        }
    }

    #[inline(always)]
    pub fn try_with_custom_namespace<'a>(namespace: &'a str, path: &'a str) -> Result<Self, AssetLocationError<'a>> {
        Self::try_new(namespace, path)
    }

    #[inline(always)]
    pub fn try_with_default_namespace(path: &str) -> Result<Self, AssetLocationError<'_>> {
        Self::try_new(Self::DEFAULT_NAMESPACE, path)
    }

    #[inline(always)]
    fn try_new<'a>(namespace: &'a str, path: &'a str) -> Result<Self, AssetLocationError<'a>> {
        Ok(Self {
            namespace: Self::compute_namespace(namespace)?,
            path: Self::compute_path(path)?,
        })
    }

    #[inline(always)]
    pub const fn parse(location: &'static str) -> Self {
        let bytes = location.as_bytes();

        let mut i = 0;
        let mut has_separator = false;

        while i < bytes.len() {
            if bytes[i] == Self::SEPARATOR {
                has_separator = true;
                break;
            }

            i += 1;
        }

        unsafe {
            let namespace = if has_separator {
                let slice = slice::from_raw_parts(bytes.as_ptr(), i);

                from_utf8_unchecked(slice)
            } else { Self::DEFAULT_NAMESPACE };

            let path = if has_separator {
                let slice = slice::from_raw_parts(bytes.as_ptr().add(i + 1), bytes.len() - i - 1);

                from_utf8_unchecked(slice)
            } else { location };

            Self::new(namespace, path)
        }
    }

    #[inline(always)]
    pub const fn with_custom_namespace(namespace: &'static str, path: &'static str) -> Self {
        Self::new(namespace, path)
    }

    #[inline(always)]
    pub const fn with_default_namespace(path: &'static str) -> Self {
        Self::new(Self::DEFAULT_NAMESPACE, path)
    }

    #[inline]
    const fn new(namespace: &'static str, path: &'static str) -> Self {
        assert!(Self::can_use_namespace(namespace), "Invalid namespace name");
        assert!(Self::can_use_path(path), "Invalid path name");

        Self { namespace: Cow::Borrowed(namespace), path: Cow::Borrowed(path) }
    }

    #[inline(always)]
    pub fn suffix(&self, suffix: &str) -> Self {
        self.try_suffixing(suffix).unwrap()
    }

    #[inline(always)]
    pub fn prefix(&self, prefix: &str) -> Self {
        self.try_prefixing(prefix).unwrap()
    }

    #[inline(always)]
    pub fn try_suffixing<'a>(&'a self, suffix: &'a str) -> Result<Self, AssetLocationError<'a>> {
        if !Self::can_use_path(suffix) {
            return Err(AssetLocationError::IllegalSuffix(suffix));
        }

        Ok(Self {
            namespace: self.namespace.clone(),
            path: Cow::Owned(format!("{}{}", self.path, suffix)),
        })
    }

    #[inline(always)]
    pub fn try_prefixing<'a>(&'a self, prefix: &'a str) -> Result<Self, AssetLocationError<'a>> {
        if !Self::can_use_path(prefix) {
            return Err(AssetLocationError::IllegalPrefix(prefix));
        }

        Ok(Self {
            namespace: self.namespace.clone(),
            path: Cow::Owned(format!("{}{}", prefix, self.path)),
        })
    }

    #[inline(always)]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    #[inline(always)]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[inline]
    fn compute_namespace(str: &str) -> Result<Cow<'static, str>, AssetLocationError<'_>> {
        if !Self::can_use_namespace(str) {
            return Err(AssetLocationError::IllegalNamespace(str));
        }

        Ok(Cow::Owned(str.to_owned()))
    }

    #[inline]
    fn compute_path(str: &str) -> Result<Cow<'static, str>, AssetLocationError<'_>> {
        if !Self::can_use_path(str) {
            return Err(AssetLocationError::IllegalPath(str));
        }

        Ok(Cow::Owned(str.to_owned()))
    }

    #[inline(always)]
    const fn can_use_namespace(namespace: &str) -> bool {
        let bytes = namespace.as_bytes();

        let mut i = 0;
        let mut valid = true;

        while i < bytes.len() {
            if !Self::is_valid_namespace_byte(&bytes[i]) {
                valid = false;
                break;
            }

            i += 1;
        }

        valid
    }

    #[inline(always)]
    const fn can_use_path(path: &str) -> bool {
        let bytes = path.as_bytes();

        let mut i = 0;
        let mut valid = true;

        while i < bytes.len() {
            if !Self::is_valid_path_byte(&bytes[i]) {
                valid = false;
                break;
            }

            i += 1;
        }

        valid
    }

    #[inline(always)]
    const fn is_valid_namespace_byte(byte: &u8) -> bool {
        byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'-'
    }

    #[inline(always)]
    const fn is_valid_path_byte(byte: &u8) -> bool {
        byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'-' || *byte == b'/'
    }
}

pub struct AssetLocationVisitor;

impl AssetLocationVisitor {
    #[inline(always)]
    fn parse_visited(v: &str) -> Result<AssetLocation, AssetLocationError<'_>> {
        match v.split_once(':') {
            None => Err(AssetLocationError::IllegalFormat),
            Some((namespace, path)) => {
                if path.contains(':') {
                    return Err(AssetLocationError::IllegalFormat);
                }

                AssetLocation::try_new(namespace, path)
            }
        }
    }
}

impl<'de> de::Visitor<'de> for AssetLocationVisitor {
    type Value = AssetLocation;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("A valid AssetLocation")
    }

    #[inline(always)]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::parse_visited(v).map_err(E::custom)
    }

    #[inline(always)]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::parse_visited(v).map_err(E::custom)
    }
}

pub enum AssetLocationError<'a> {
    IllegalNamespace(&'a str),
    IllegalPath(&'a str),
    IllegalPrefix(&'a str),
    IllegalSuffix(&'a str),
    IllegalFormat,
    Custom(String),
}

impl AssetLocationError<'_> {
    fn format_error(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLocationError::IllegalNamespace(n) => {
                f.write_str("Illegal namespace: ")?;
                f.write_str(n)
            }
            AssetLocationError::IllegalPath(s) => {
                f.write_str("Illegal path: ")?;
                f.write_str(s)
            }
            AssetLocationError::IllegalPrefix(s) => {
                f.write_str("Illegal prefix: ")?;
                f.write_str(s)
            }
            AssetLocationError::IllegalSuffix(s) => {
                f.write_str("Illegal suffix: ")?;
                f.write_str(s)
            }
            AssetLocationError::IllegalFormat => {
                f.write_str("Illegal AssetLocation format (expected 'namespace:path')")
            }
            AssetLocationError::Custom(msg) => {
                f.write_str("Unexpected AssetLocation error: ")?;
                f.write_str(msg)
            }
        }
    }
}

impl de::Error for AssetLocationError<'_> {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl Error for AssetLocationError<'_> {}

impl Debug for AssetLocationError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format_error(f)
    }
}

impl Display for AssetLocationError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format_error(f)
    }
}
