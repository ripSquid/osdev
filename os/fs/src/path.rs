use core::{borrow::Borrow, ops::Deref};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Clone)]
pub struct Path(pub(crate) String);
impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}


pub struct PathStr(str);

impl PathStr {
    /// This is taken almost word for word of the standard library
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &PathStr {
        unsafe { &*(s.as_ref() as *const str as *const PathStr) }
    }
}

impl AsRef<PathStr> for Path {
    fn as_ref(&self) -> &PathStr {
        PathStr::new(self.0.as_str())
    }
}

impl PathStr {
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split("/")
    }
    pub fn file_extension(&self) -> Option<&str> {
        self.components().last().map(|i| i.split('.').last()).flatten()
    }
    pub fn file_name(&self) -> Option<&Self> {
        self.components().last().map(|i| Self::new(i))
    }
}

impl Borrow<PathStr> for Path {
    fn borrow(&self) -> &PathStr {
        self.as_path_str()
    }
}
impl Deref for Path {
    type Target = PathStr;

    fn deref(&self) -> &Self::Target {
        self.as_path_str()
    }
    
}

impl Path {
    pub fn as_path_str(&self) -> &PathStr {
        self.as_ref()
    }
    pub fn parent(&self) -> Option<Self> {
        let clean = self.clone().clean();
        let segments: Vec<_> = clean.components().collect();
        if segments.len() < 1 {
            return None;
        }
        Some(Self::from_segments(&segments[..segments.len() - 1]))
    }
    pub fn pop(&mut self) -> Option<Path> {
        let clean = self.clone().clean();
        let segments: Vec<_> = clean.components().collect();
        if segments.len() < 1 {
            return None;
        }
        *self = Self::from_segments(&segments[..segments.len() - 1]);
        Some(Self::from(*segments.last()?))
    }
    pub fn from_segments(items: &[&str]) -> Self {
        Self(items.iter().fold(String::new(), |mut x, i| {
            x.push('/');
            x.push_str(i);
            x
        }))
    }
    pub fn new() -> Self {
        Self(String::new())
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn append<A: AppendsPath>(mut self, path: &A) -> Self {
        self.0.push_str("/");
        self.0.push_str(path.to_str());
        self
    }

    pub fn clean(self) -> Self {
        let mut new = Vec::new();
        let sections = self.0.split('/');
        for section in sections {
            match section {
                "" => continue,
                "." => continue,
                ".." => {
                    new.pop();
                    continue;
                }
                _ => (),
            }
            new.push(section);
        }
        let mut finale = String::new();
        for part in new.into_iter() {
            finale.push('/');
            finale.push_str(part);
        }
        Self(finale)
    }
    pub fn add_extension(mut self, extension: &str) -> Self {
        self.0.push('.');
        self.0.push_str(extension);
        self
    }
}
pub trait AppendsPath {
    fn to_str(&self) -> &str;
}
impl<T: AsRef<str>> AppendsPath for T {
    fn to_str(&self) -> &str {
        self.as_ref()
    }
}
impl AppendsPath for Path {
    fn to_str(&self) -> &str {
        self.0.as_str()
    }
}
impl From<String> for Path {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}