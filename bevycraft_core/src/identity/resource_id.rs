use std::borrow::Cow;
use std::fmt::*;
use std::hash::*;
use std::str::from_utf8_unchecked;
use bevy::platform::hash::FixedHasher;

pub struct ResourceId {
    hash: u64,
    int: usize,
    vec: Vec<u8>,
}

impl ResourceId {
    pub const DEFAULT_NAMESPACE: &'static str = "bevycraft";

    #[inline(always)]
    pub unsafe fn new_unchecked(namespace: &str, path: &str) -> Self {
        let int = namespace.len();

        let mut vec = Vec::from(namespace);
        vec.extend_from_slice(path.as_bytes());

        let hash = Self::compute_hash(namespace, path);

        Self {
            hash,
            int,
            vec,
        }
    }

    #[inline]
    fn new(namespace: &str, path: &str) -> Self {
        Self::assert_namespace(namespace);
        Self::assert_path(path);

        unsafe { Self::new_unchecked(namespace, path) }
    }

    #[inline]
    pub fn default_namespace(path: &str) -> Self {
        Self::new(Self::DEFAULT_NAMESPACE, path)
    }

    #[inline]
    pub fn custom_namespace(namespace: &str, path: &str) -> Self {
        Self::new(namespace, path)
    }

    #[inline]
    pub fn parse(location: &str) -> Self {
        match location.split_once(":") {
            None =>
                Self::new(Self::DEFAULT_NAMESPACE, location),
            Some((namespace, path)) =>
                Self::new(namespace, path),
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.vec.as_slice()
    }

    #[inline]
    fn compute_hash(namespace: &str, path: &str) -> u64 {
        let mut hasher = FixedHasher::default().build_hasher();

        namespace.hash(&mut hasher);
        path.hash(&mut hasher);

        hasher.finish()
    }

    #[inline]
    fn assert_namespace(namespace: &str) {
        assert!(namespace.bytes().all(Self::valid_namespace_byte), "{}", format!("Invalid byte found in namespace '{}'", namespace));
    }

    #[inline]
    fn assert_path(path: &str) {
        assert!(path.bytes().all(Self::valid_path_byte), "{}", format!("Invalid byte found in path '{}'", path));
    }

    #[inline]
    const fn valid_namespace_byte(byte: u8) -> bool {
        matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_')
    }

    #[inline]
    const fn valid_path_byte(byte: u8) -> bool {
        matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/')
    }
}

impl Clone for ResourceId {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::new_unchecked(self.namespace(), self.path()) }
    }
}

impl PartialEq for ResourceId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.namespace() == other.namespace()
            && self.path() == other.path()
    }
}

impl Eq for ResourceId {}

impl Hash for ResourceId {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
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

impl Debug for ResourceId {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("ResourceId")
            .field("namespace", &self.namespace())
            .field("path", &self.path())
            .finish()
    }
}

impl NamespacedIdentifier for ResourceId {
    #[inline]
    fn namespace(&self) -> &str {
        unsafe { from_utf8_unchecked(&self.vec[..self.int]) }
    }

    #[inline]
    fn path(&self) -> &str {
        unsafe { from_utf8_unchecked(&self.vec[self.int..]) }
    }
}

pub trait NamespacedIdentifier {
    fn namespace(&self) -> &str;

    fn path(&self) -> &str;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Namespace<'a>(Cow<'a, str>);

impl<'a> Namespace<'a> {
    pub const unsafe fn new_unchecked(namespace: &'a str) -> Self {
        Self(Cow::Borrowed(namespace))
    }

    #[inline]
    pub const fn new(namespace: &'a str) -> Self {
        debug_assert!(Self::valid_namespace(namespace), "Invalid byte found in namespace");

        unsafe { Self::new_unchecked(namespace) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
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

impl<'a> From<&'a str> for Namespace<'a> {
    #[inline]
    fn from(namespace: &'a str) -> Self {
        Self::new(namespace)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Path<'a>(Cow<'a, str>);

impl<'a> Path<'a> {
    pub const unsafe fn new_unchecked(path: &'a str) -> Self {
        Self(Cow::Borrowed(path))
    }

    #[inline]
    pub const fn new(path: &'a str) -> Self {
        debug_assert!(Self::valid_path(path), "Invalid byte found in path");

        unsafe { Self::new_unchecked(path) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }


    #[inline]
    #[must_use]
    pub fn prefix(&self, prefix: &'a str) -> Self {
        debug_assert!(Self::valid_path(prefix), "Invalid byte found in prefix");

        match &self.0 {
            Cow::Borrowed(b) => {
                Self(Cow::Owned(format!("{}{}", prefix, b)))
            }
            Cow::Owned(o) => {
                Self(Cow::Owned(format!("{}{}", prefix, o)))
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn suffix(&self, suffix: &'a str) -> Self {
        debug_assert!(Self::valid_path(suffix), "Invalid byte found in suffix");

        match &self.0 {
            Cow::Borrowed(b) => {
                Self(Cow::Owned(format!("{}{}", b, suffix)))
            }
            Cow::Owned(o) => {
                Self(Cow::Owned(format!("{}{}", o, suffix)))
            }
        }
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

impl Display for Path<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.as_str())
    }
}

impl<'a> From<&'a str> for Path<'a> {
    #[inline]
    fn from(path: &'a str) -> Self {
        Self::new(path)
    }
}