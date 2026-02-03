use std::borrow::Cow;
use std::fmt::*;
use std::hash::*;
use std::ops::Deref;
use std::slice::from_raw_parts;
use std::str::FromStr;
use std::str::from_utf8_unchecked;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ResourceId {
    namespace: Namespace,
    path: Path,
}

impl ResourceId {
    pub const DEFAULT_NAMESPACE: Namespace = Namespace::new_static("bevycraft");

    #[inline]
    pub const unsafe fn new_static_unchecked(namespace: &'static str, path: &'static str) -> Self {
        Self {
            namespace: Namespace::new_static_unchecked(namespace),
            path: Path::new_static_unchecked(path),
        }
    }

    #[inline]
    pub const fn new_static(namespace: &'static str, path: &'static str) -> Self {
        Self {
            namespace: Namespace::new_static(namespace),
            path: Path::new_static(path),
        }
    }

    #[inline]
    pub const fn default_namespace_static(path: &'static str) -> Self {
        Self {
            namespace: Self::DEFAULT_NAMESPACE,
            path: Path::new_static(path),
        }
    }

    #[inline]
    pub const fn custom_namespace_static(namespace: &'static str, path: &'static str) -> Self {
        Self {
            namespace: Namespace::new_static(namespace),
            path: Path::new_static(path),
        }
    }

    #[inline]
    pub const fn parse_static(location: &'static str) -> Self {
        let bytes = location.as_bytes();
        let mut i = 0usize;
        let mut indent = 0usize;

        while i < bytes.len() {
            if bytes[i] == b':' {
                indent = i;
                break;
            }

            i += 1;
        }

        if i != 0 {
            let ptr = location.as_ptr();

            unsafe {
                Self::custom_namespace_static(
                    from_utf8_unchecked(from_raw_parts(ptr, indent)),
                    from_utf8_unchecked(from_raw_parts(
                        ptr.add(indent + 1),
                        location.len() - indent - 1,
                    )),
                )
            }
        } else {
            Self::default_namespace_static(location)
        }
    }

    #[inline]
    pub unsafe fn new_unchecked(namespace: &str, path: &str) -> Self {
        Self {
            namespace: Namespace::new_unchecked(namespace),
            path: Path::new_unchecked(path),
        }
    }

    #[inline]
    fn new(namespace: &str, path: &str) -> Self {
        Self {
            namespace: Namespace::new(namespace),
            path: Path::new(path),
        }
    }

    #[inline]
    pub fn default_namespace(path: &str) -> Self {
        Self {
            namespace: Self::DEFAULT_NAMESPACE,
            path: Path::new(path),
        }
    }

    #[inline]
    pub fn custom_namespace(namespace: &str, path: &str) -> Self {
        Self::new(namespace, path)
    }

    #[inline]
    pub fn parse(location: &str) -> Self {
        match location.split_once(":") {
            None => Self::default_namespace(location),
            Some((namespace, path)) => Self::new(namespace, path),
        }
    }
}

impl Display for ResourceId {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.namespace())?;
        f.write_char(':')?;
        f.write_str(self.path())
    }
}

impl NamespacedIdentifier for ResourceId {
    #[inline]
    fn namespace(&self) -> &str {
        &self.namespace
    }

    #[inline]
    fn path(&self) -> &str {
        &self.path
    }
}

pub trait NamespacedIdentifier {
    fn namespace(&self) -> &str;

    fn path(&self) -> &str;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct Namespace(Cow<'static, str>);

impl Namespace {
    pub const unsafe fn new_static_unchecked(namespace: &'static str) -> Self {
        Self(Cow::Borrowed(namespace))
    }

    #[inline]
    pub unsafe fn new_unchecked(namespace: &str) -> Self {
        Self(Cow::Owned(namespace.to_owned()))
    }

    #[inline]
    pub const fn new_static(namespace: &'static str) -> Self {
        debug_assert!(
            Self::valid_namespace(namespace),
            "Invalid byte found in namespace"
        );

        unsafe { Self::new_static_unchecked(namespace) }
    }

    #[inline]
    pub fn new(namespace: &str) -> Self {
        debug_assert!(
            namespace.bytes().all(Self::valid_byte),
            "Invalid byte found in namespace"
        );

        unsafe { Self::new_unchecked(namespace) }
    }

    #[inline]
    const fn valid_namespace(s: &str) -> bool {
        let bytes = s.as_bytes();
        let mut i = 0usize;

        while i < bytes.len() {
            if !Self::valid_byte(bytes[i]) {
                return false;
            }

            i += 1;
        }

        true
    }

    const fn valid_byte(byte: u8) -> bool {
        matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_')
    }
}

impl Deref for Namespace {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(&self)
    }
}

impl From<String> for Namespace {
    fn from(value: String) -> Self {
        debug_assert!(
            value.bytes().all(Self::valid_byte),
            "Invalid byte found in namespace"
        );

        Self(Cow::Owned(value))
    }
}

impl FromStr for Namespace {
    type Err = ();

    #[inline]
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct Path(Cow<'static, str>);

impl Path {
    pub const unsafe fn new_static_unchecked(path: &'static str) -> Self {
        Self(Cow::Borrowed(path))
    }

    #[inline]
    pub unsafe fn new_unchecked(path: &str) -> Self {
        Self(Cow::Owned(path.to_owned()))
    }

    #[inline]
    pub const fn new_static(path: &'static str) -> Self {
        debug_assert!(Self::valid_path(path), "Invalid byte found in path");

        unsafe { Self::new_static_unchecked(path) }
    }

    #[inline]
    pub fn new(path: &str) -> Self {
        debug_assert!(
            path.bytes().all(Self::valid_byte),
            "Invalid byte found in path"
        );

        unsafe { Self::new_unchecked(path) }
    }

    #[inline]
    #[must_use]
    pub fn prefix(&mut self, prefix: &str) -> &Self {
        debug_assert!(
            prefix.bytes().all(Self::valid_byte),
            "Invalid byte found in prefix"
        );

        self.0.to_mut().insert_str(0, prefix);

        self
    }

    #[inline]
    #[must_use]
    pub fn suffix(&mut self, suffix: &str) -> &Self {
        debug_assert!(
            suffix.bytes().all(Self::valid_byte),
            "Invalid byte found in suffix"
        );

        self.0.to_mut().push_str(suffix);

        self
    }

    #[inline]
    #[must_use]
    pub fn clone_prefixed(&self, prefix: &str) -> Self {
        debug_assert!(
            prefix.bytes().all(Self::valid_byte),
            "Invalid byte found in prefix"
        );

        let mut cloned = self.clone();

        cloned.0.to_mut().insert_str(0, prefix);

        cloned
    }

    #[inline]
    #[must_use]
    pub fn clone_suffixed(&self, suffix: &str) -> Self {
        debug_assert!(
            suffix.bytes().all(Self::valid_byte),
            "Invalid byte found in suffix"
        );

        let mut cloned = self.clone();

        cloned.0.to_mut().push_str(suffix);

        cloned
    }

    #[inline]
    const fn valid_path(s: &str) -> bool {
        let bytes = s.as_bytes();
        let mut i = 0usize;

        while i < bytes.len() {
            if !Self::valid_byte(bytes[i]) {
                return false;
            }

            i += 1;
        }

        true
    }

    const fn valid_byte(byte: u8) -> bool {
        matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/')
    }
}

impl Deref for Path {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(&self)
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        debug_assert!(
            value.bytes().all(Self::valid_byte),
            "Invalid byte found in path"
        );

        Self(Cow::Owned(value))
    }
}

impl FromStr for Path {
    type Err = ();

    #[inline]
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}
