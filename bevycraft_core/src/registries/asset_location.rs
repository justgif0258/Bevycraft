use {
    bitcode::{Decode, Encode},
    serde::*,
    std::{
        error::Error,
        fmt::{Debug, Display, Formatter, Write},
        hash::Hash,
    },
};

#[derive(Decode, Encode, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetLocation {
    namespace: String,
    path: String,
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

impl Debug for AssetLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistrationId")
            .field("namespace", &self.namespace)
            .field("path", &self.path)
            .finish()
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
            namespace: String::from(Self::DEFAULT_NAMESPACE),
            path: String::from("air"),
        }
    }
}

impl AssetLocation {
    pub const DEFAULT_NAMESPACE: &'static str = "bevycraft";

    #[inline(always)]
    fn try_new<'a>(namespace: &'a str, path: &'a str) -> Result<Self, AssetLocationError<'a>> {
        Ok(Self {
            namespace: Self::compute_namespace(namespace)?,
            path: Self::compute_path(path)?,
        })
    }

    #[inline(always)]
    pub fn try_with_default_namespace(path: &str) -> Result<Self, AssetLocationError<'_>> {
        Ok(Self {
            namespace: Self::DEFAULT_NAMESPACE.to_string(),
            path: Self::compute_path(path)?,
        })
    }

    #[inline(always)]
    pub fn try_with_custom_namespace<'a>(
        namespace: &'a str,
        path: &'a str,
    ) -> Result<Self, AssetLocationError<'a>> {
        Self::try_new(namespace, path)
    }

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
    pub fn with_default_namespace(path: &str) -> Self {
        Self::try_with_default_namespace(path).unwrap()
    }

    #[inline(always)]
    pub fn with_custom_namespace(namespace: &str, path: &str) -> Self {
        Self::try_new(namespace, path).unwrap()
    }

    #[inline(always)]
    pub fn parse(location: &str) -> Self {
        Self::try_parse(location).unwrap()
    }

    #[inline(always)]
    pub fn try_prefixing<'a>(
        &'a self,
        prefix: &'a str,
    ) -> Result<Self, AssetLocationError<'a>> {
        if !Self::can_use_path(prefix) {
            return Err(AssetLocationError::IllegalPrefix(prefix));
        }

        Ok(Self {
            namespace: self.namespace.clone(),
            path: format!("{}{}", prefix, &self.path),
        })
    }

    #[inline(always)]
    pub fn try_suffixing<'a>(
        &'a self,
        suffix: &'a str,
    ) -> Result<Self, AssetLocationError<'a>> {
        if !Self::can_use_path(suffix) {
            return Err(AssetLocationError::IllegalSuffix(suffix));
        }

        Ok(Self {
            namespace: self.namespace.clone(),
            path: format!("{}{}", &self.path, suffix),
        })
    }

    #[inline(always)]
    pub fn prefix(&self, prefix: &str) -> Self {
        self.try_prefixing(prefix).unwrap()
    }

    #[inline(always)]
    pub fn suffix(&self, suffix: &str) -> Self {
        self.try_suffixing(suffix).unwrap()
    }

    #[inline(always)]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    #[inline(always)]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[inline(always)]
    fn compute_namespace(str: &str) -> Result<String, AssetLocationError<'_>> {
        if !Self::can_use_namespace(str) {
            return Err(AssetLocationError::IllegalNamespace(str));
        }

        Ok(str.to_string())
    }

    #[inline(always)]
    fn compute_path(str: &str) -> Result<String, AssetLocationError<'_>> {
        if !Self::can_use_path(str) {
            return Err(AssetLocationError::IllegalPath(str));
        }

        Ok(str.to_string())
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
    fn is_valid_namespace_byte(byte: &u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == &b'_' || byte == &b'-'
    }

    #[inline(always)]
    fn is_valid_path_byte(byte: &u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == &b'_' || byte == &b'-' || byte == &b'/'
    }
}

pub struct AssetLocationVisitor;

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
        AssetLocation::try_parse(v).map_err(E::custom)
    }

    #[inline(always)]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.split_once(':') {
            None => Err(E::custom(AssetLocationError::IllegalFormat)),
            Some((namespace, path)) => {
                if path.contains(':') {
                    return Err(E::custom(AssetLocationError::IllegalFormat));
                }

                AssetLocation::try_new(namespace, path).map_err(E::custom)
            }
        }
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
