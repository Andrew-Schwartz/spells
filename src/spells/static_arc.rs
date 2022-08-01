use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Eq, PartialEq, Debug, Ord, PartialOrd, Hash)]
pub enum StArc<T: ?Sized + 'static> {
    Static(&'static T),
    Arc(Arc<T>),
}

impl<T: ?Sized> Deref for StArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static(t) => t,
            Self::Arc(t) => t,
        }
    }
}

impl<T: ?Sized> Clone for StArc<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(t) => Self::Static(*t),
            Self::Arc(t) => Self::Arc(t.clone()),
        }
    }
}

impl<T: ?Sized> From<&'static T> for StArc<T> {
    fn from(t: &'static T) -> Self {
        Self::Static(t)
    }
}

impl<T: ?Sized> From<Arc<T>> for StArc<T> {
    fn from(t: Arc<T>) -> Self {
        Self::Arc(t)
    }
}

impl<'a, T: ?Sized> From<&'a Arc<T>> for StArc<T> {
    fn from(t: &'a Arc<T>) -> Self {
        Self::Arc(Arc::clone(t))
    }
}

impl<'de, T: ?Sized> Deserialize<'de> for StArc<T>
    where Arc<T>: Deserialize<'de> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::Arc(<Arc<T>>::deserialize(d)?))
    }
}

impl<T: ?Sized + Serialize> Serialize for StArc<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Static(t) => t.serialize(s),
            Self::Arc(t) => t.serialize(s),
        }
    }
}

impl<'a, T: ?Sized + PartialEq> PartialEq<&'a T> for StArc<T> {
    fn eq(&self, other: &&'a T) -> bool {
        **self == **other
    }
}

impl<T: ?Sized> Display for StArc<T> where T: Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StArc::Static(t) => t.fmt(f),
            StArc::Arc(t) => (&**t).fmt(f),
        }
    }
}
