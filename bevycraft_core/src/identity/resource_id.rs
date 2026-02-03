use bevy::platform::hash::FixedHasher;
use std::borrow::Cow;
use std::fmt::*;
use std::hash::*;
use std::ops::Deref;
use std::str::{from_utf8_unchecked, FromStr};

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

        Self { hash, int, vec }
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
            None => Self::new(Self::DEFAULT_NAMESPACE, location),
            Some((namespace, path)) => Self::new(namespace, path),
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
        assert!(
            namespace.bytes().all(Self::valid_namespace_byte),
            "{}",
            format!("Invalid byte found in namespace '{}'", namespace)
        );
    }

    #[inline]
    fn assert_path(path: &str) {
        assert!(
            path.bytes().all(Self::valid_path_byte),
            "{}",
            format!("Invalid byte found in path '{}'", path)
        );
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
        self.namespace() == other.namespace() && self.path() == other.path()
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
