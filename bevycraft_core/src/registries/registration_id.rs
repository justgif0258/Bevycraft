use {
    std::{
        error::Error,
        fmt::{Debug, Display, Formatter, Write},
        hash::{BuildHasher, Hash, Hasher},
        mem::transmute,
        sync::OnceLock,
    },
    bevy::platform::collections::HashMap,
    lasso::{Capacity, Spur, ThreadedRodeo},
    rapidhash::fast::RandomState,
    serde::*,
};

pub type RegistrationMap<V> = HashMap<RegistrationId, V, IdentityHasherBuilder>;

pub type RapidThreadedRodeo = ThreadedRodeo<Spur, RandomState>;

static GLOBAL_INTERN: OnceLock<RapidThreadedRodeo> = OnceLock::new();

#[ctor::ctor]
fn init_interner() {
    let interner = ThreadedRodeo::with_capacity_and_hasher(
        Capacity::for_strings(256),
        RandomState::new(),
    );

    interner.get_or_intern(RegistrationId::DEFAULT_NAMESPACE);

    GLOBAL_INTERN.set(interner)
        .expect("Failed to init interner");
}

#[repr(C)]
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct RegistrationId {
    namespace   : Spur,
    path        : Spur,
}

impl<'de> Deserialize<'de> for RegistrationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        RegistrationId::try_from(deserializer.deserialize_str(RegistrationIdVisitor)?)
            .map_err(de::Error::custom)
    }
}

impl Serialize for RegistrationId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl TryFrom<&str> for RegistrationId {
    type Error = RegistrationIdError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_parsing(value)
    }
}

impl TryFrom<String> for RegistrationId {
    type Error = RegistrationIdError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_parsing(value.as_str())
    }
}

impl Hash for RegistrationId {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash_u64())
    }
}

impl Debug for RegistrationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistrationId")
            .field("namespace", &self.namespace())
            .field("path", &self.path())
            .finish()
    }
}

impl Display for RegistrationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace())?;
        f.write_char(':')?;
        f.write_str(&self.path())
    }
}

impl RegistrationId {
    pub const DEFAULT_NAMESPACE: &'static str = "bevycraft";

    #[inline]
    pub fn try_parsing(raw: &str) -> Result<Self, RegistrationIdError> {
        match raw.split_once(':') {
            None => Self::try_with_default_namespace(raw),
            Some((n, p)) => Self::try_new(n, p),
        }
    }

    #[inline]
    pub fn try_with_custom_namespace(
        namespace: &str,
        path: &str,
    ) -> Result<Self, RegistrationIdError> {
        Self::try_new(namespace, path)
    }

    #[inline]
    pub fn try_with_default_namespace(path: &str) -> Result<Self, RegistrationIdError> {
        if let Some(interned) = try_get_spur(path) {
            return Ok(
                Self {
                    namespace: Self::compute_namespace(Self::DEFAULT_NAMESPACE),
                    path: interned,
                }
            )
        }

        if !Self::can_use_path(path) {
            return Err(RegistrationIdError::IllegalPath(path.to_string()));
        }

        unsafe { Ok(Self::new_unchecked(Self::DEFAULT_NAMESPACE, path)) }
    }

    #[inline]
    fn try_new(
        namespace: &str,
        path: &str,
    ) -> Result<Self, RegistrationIdError> {
        if !Self::can_use_namespace(namespace) {
            return Err(RegistrationIdError::IllegalNamespace(namespace.to_string()))
        }

        if !Self::can_use_path(path) {
            return Err(RegistrationIdError::IllegalPath(path.to_string()))
        }

        unsafe { Ok(Self::new_unchecked(namespace, path)) }
    }

    #[inline]
    pub fn parse(raw: &str) -> Self {
        match raw.split_once(':') {
            None => Self::with_default_namespace(raw),
            Some((n, p)) => Self::new(n, p),
        }
    }

    #[inline]
    pub fn with_custom_namespace(
        namespace: &str,
        path: &str,
    ) -> Self {
        Self::new(namespace, path)
    }

    #[inline]
    pub fn with_default_namespace(path: &str) -> Self {
        Self {
            namespace: Self::compute_namespace(Self::DEFAULT_NAMESPACE),
            path: Self::compute_path(path),
        }
    }

    #[inline]
    fn new(
        namespace: &str,
        path: &str,
    ) -> Self {
        Self {
            namespace: Self::compute_namespace(namespace),
            path: Self::compute_path(path),
        }
    }

    #[inline]
    pub unsafe fn new_unchecked(
        namespace: &str,
        path: &str,
    ) -> Self {
        Self {
            namespace: get_or_intern_str(namespace),
            path: get_or_intern_str(path),
        }
    }

    #[inline]
    pub fn path(&self) -> &str {
        resolve_spur(&self.path)
    }

    #[inline]
    pub fn namespace(&self) -> &str {
        resolve_spur(&self.namespace)
    }

    #[inline]
    pub const fn hash_u64(&self) -> u64 {
        unsafe { transmute::<_, u64>((self.namespace, self.path)) }
    }

    #[inline]
    fn compute_path(str: &str) -> Spur {
        if let Some(interned) = try_get_spur(str) {
            return interned;
        }

        assert!(Self::can_use_path(str), "Illegal path: {}", str);

        get_or_intern_str(str)
    }

    #[inline]
    fn compute_namespace(str: &str) -> Spur {
        if let Some(interned) = try_get_spur(str) {
            return interned;
        }

        assert!(Self::can_use_namespace(str), "Illegal namespace: {}", str);

        get_or_intern_str(str)
    }

    #[inline]
    fn can_use_path(path: &str) -> bool {
        path.as_bytes()
            .iter()
            .all(Self::is_valid_path_byte)
    }

    #[inline]
    fn can_use_namespace(namespace: &str) -> bool {
        namespace.as_bytes()
            .iter()
            .all(Self::is_valid_namespace_byte)
    }

    #[inline]
    const fn is_valid_path_byte(byte: &u8) -> bool {
        matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'/' | b'.')
    }

    #[inline]
    const fn is_valid_namespace_byte(byte: &u8) -> bool {
        matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-')
    }
}

pub struct RegistrationIdVisitor;

impl<'de> de::Visitor<'de> for RegistrationIdVisitor {
    type Value = &'de str;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("A valid string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }
}

pub enum RegistrationIdError {
    IllegalNamespace(String),
    IllegalPath(String),
    IllegalFormat,
    Custom(String)
}

impl de::Error for RegistrationIdError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display
    {
        Self::Custom(msg.to_string())
    }
}

impl Error for RegistrationIdError {}

impl Debug for RegistrationIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for RegistrationIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrationIdError::IllegalNamespace(n) => {
                f.write_str("Illegal namespace: ")?;
                f.write_str(n)
            },
            RegistrationIdError::IllegalPath(s) => {
                f.write_str("Illegal path: ")?;
                f.write_str(s)
            }
            RegistrationIdError::IllegalFormat => {
                f.write_str("Illegal RegistrationId format (expected 'namespace:path')")
            },
            RegistrationIdError::Custom(msg) => {
                f.write_str(msg)
            },
        }
    }
}

pub struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    #[allow(unused_variables)]
    fn write(&mut self, bytes: &[u8]) {
        panic!("IdentityHasher does not allow writing bytes")
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i
    }
}

#[derive(Default)]
pub struct IdentityHasherBuilder;

impl BuildHasher for IdentityHasherBuilder {
    type Hasher = IdentityHasher;

    fn build_hasher(&self) -> Self::Hasher {
        IdentityHasher(0)
    }
}

#[inline]
fn resolve_spur(spur: &Spur) -> &str {
    unsafe {
        GLOBAL_INTERN.get()
            .unwrap_unchecked()
            .resolve(spur)
    }
}

#[inline]
fn try_get_spur(str: &str) -> Option<Spur> {
    unsafe {
        GLOBAL_INTERN.get()
            .unwrap_unchecked()
            .get(str)
    }
}

#[inline]
fn get_or_intern_str(str: &str) -> Spur {
    unsafe {
        GLOBAL_INTERN.get()
            .unwrap_unchecked()
            .get_or_intern(str)
    }
}
