use {
    bevy::asset::AssetPath,
    serde::*,
    std::{
        fmt::{Debug, Display, Formatter, Write},
        hash::Hash,
    },
    thiserror::Error,
};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetLocation {
    namespace: Box<str>,
    path: Box<str>,
}

impl Default for AssetLocation {
    #[inline(always)]
    fn default() -> Self {
        Self {
            namespace: Box::from(Self::DEFAULT_NAMESPACE),
            path: Box::from("air"),
        }
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

impl TryFrom<String> for AssetLocation {
    type Error = AssetLocationError;

    #[inline(always)]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.split_once(Self::SEPARATOR) {
            None => {
                if !Self::can_use_path(&value) {
                    return Err(AssetLocationError::IllegalPath(value));
                }

                Ok(Self {
                    namespace: Self::DEFAULT_NAMESPACE.into(),
                    path: value.into_boxed_str(),
                })
            }
            Some((n, p)) => Self::try_with_custom_namespace(n, p),
        }
    }
}

impl<'a> TryFrom<&'a str> for AssetLocation {
    type Error = AssetLocationError;

    #[inline(always)]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::try_parsing(value)
    }
}

impl AssetLocation {
    pub const DEFAULT_NAMESPACE: &'static str = "bevycraft";

    const SEPARATOR: char = ':';

    #[inline]
    pub fn new_unchecked(namespace: &str, path: &str) -> Self {
        Self {
            namespace: Box::from(namespace),
            path: Box::from(path),
        }
    }

    #[inline]
    pub fn parse(location: &str) -> Self {
        Self::try_parsing(location).unwrap()
    }

    #[inline]
    pub fn with_custom_namespace(namespace: &str, path: &str) -> Self {
        Self::try_with_custom_namespace(namespace, path).unwrap()
    }

    #[inline]
    pub fn with_default_namespace(path: &str) -> Self {
        Self::try_with_default_namespace(path).unwrap()
    }

    #[inline]
    pub fn try_parsing(location: &str) -> Result<Self, AssetLocationError> {
        match location.split_once(Self::SEPARATOR) {
            None => Self::try_with_default_namespace(location),
            Some((n, p)) => {
                if p.contains(Self::SEPARATOR) {
                    return Err(AssetLocationError::InvalidFormat);
                }

                Self::try_new(n, p)
            }
        }
    }

    #[inline]
    pub fn try_with_custom_namespace(
        namespace: &str,
        path: &str,
    ) -> Result<Self, AssetLocationError> {
        Self::try_new(namespace, path)
    }

    #[inline]
    pub fn try_with_default_namespace(path: &str) -> Result<Self, AssetLocationError> {
        Self::try_new(Self::DEFAULT_NAMESPACE, path)
    }

    #[inline]
    fn try_new(namespace: &str, path: &str) -> Result<Self, AssetLocationError> {
        let namespace = {
            if !Self::can_use_namespace(namespace) {
                return Err(AssetLocationError::IllegalNamespace(namespace.to_string()));
            }

            Box::from(namespace)
        };

        let path = {
            if !Self::can_use_path(path) {
                return Err(AssetLocationError::IllegalPath(path.to_string()));
            }

            Box::from(path)
        };

        Ok(Self { namespace, path })
    }

    #[inline]
    pub fn suffix(&self, suffix: &str) -> Self {
        self.try_suffixing(suffix).unwrap()
    }

    #[inline]
    pub fn prefix(&self, prefix: &str) -> Self {
        self.try_prefixing(prefix).unwrap()
    }

    #[inline]
    pub fn try_suffixing(&self, suffix: &str) -> Result<Self, AssetLocationError> {
        if !Self::can_use_path(suffix) {
            return Err(AssetLocationError::IllegalSuffix(suffix.to_string()));
        }

        Ok(Self {
            namespace: self.namespace.clone(),
            path: [self.path.as_ref(), suffix].concat().into_boxed_str(),
        })
    }

    #[inline]
    pub fn try_prefixing(&self, prefix: &str) -> Result<Self, AssetLocationError> {
        if !Self::can_use_path(prefix) {
            return Err(AssetLocationError::IllegalPrefix(prefix.to_string()));
        }

        Ok(Self {
            namespace: self.namespace.clone(),
            path: [prefix, self.path.as_ref()].concat().into_boxed_str(),
        })
    }

    #[inline(always)]
    pub fn namespace(&self) -> &str {
        self.namespace.as_ref()
    }

    #[inline(always)]
    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    #[inline(always)]
    fn can_use_namespace(namespace: &str) -> bool {
        namespace
            .as_bytes()
            .iter()
            .all(Self::is_valid_namespace_byte)
    }

    #[inline(always)]
    fn can_use_path(path: &str) -> bool {
        path.as_bytes().iter().all(Self::is_valid_path_byte)
    }

    #[inline(always)]
    const fn is_valid_namespace_byte(byte: &u8) -> bool {
        byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'-'
    }

    #[inline(always)]
    const fn is_valid_path_byte(byte: &u8) -> bool {
        byte.is_ascii_alphanumeric()
            || *byte == b'_'
            || *byte == b'-'
            || *byte == b'/'
            || *byte == b'.'
    }
}

impl<'a> Into<AssetPath<'a>> for AssetLocation {
    #[inline(always)]
    fn into(self) -> AssetPath<'a> {
        let path = format!("{}\\{}", self.namespace, self.path).replace('/', "\\");

        AssetPath::from(path)
    }
}

impl Display for AssetLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_char(':')?;
        f.write_str(&self.path)
    }
}

pub struct AssetLocationVisitor;

impl<'de> de::Visitor<'de> for AssetLocationVisitor {
    type Value = AssetLocation;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a valid AssetLocation")
    }

    #[inline(always)]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        AssetLocation::try_parsing(v).map_err(E::custom)
    }

    #[inline(always)]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        AssetLocation::try_parsing(v).map_err(E::custom)
    }

    #[inline(always)]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        AssetLocation::try_from(v).map_err(E::custom)
    }
}

#[derive(Error, Debug)]
pub enum AssetLocationError {
    #[error("illegal namespace: `{0}`")]
    IllegalNamespace(String),

    #[error("illegal path: `{0}`")]
    IllegalPath(String),

    #[error("illegal prefix: `{0}`")]
    IllegalPrefix(String),

    #[error("illegal suffix: `{0}`")]
    IllegalSuffix(String),

    #[error("invalid format (expected `namespace:path`)")]
    InvalidFormat,

    #[error("unknown: {0}")]
    Custom(String),
}
