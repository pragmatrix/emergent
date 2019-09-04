//! A data structure that models an acyclic directed graph on which leaf
//! nodes can only be appended.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;

pub trait Chain<Link> {
    /// Returns the length of the chain.
    fn len(&self) -> usize;

    /// Computes the element based on a reference inside the chain.
    ///
    /// This function returns either reference to the element,
    /// or `None` if `link` points to the Identity element.
    fn resolve(&mut self, link: Ref<Link>) -> Option<&Link>;

    /// Returns `true` `if len() == 0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// An link chain is a chain that contains links.
#[derive(Debug)]
pub struct LinkChain<Link>(Vec<Link>);

impl<Link: Clone> Clone for LinkChain<Link> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Link: PartialEq> PartialEq for LinkChain<Link> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Link: Serialize> Serialize for LinkChain<Link> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
impl<'de, Link: Deserialize<'de>> Deserialize<'de> for LinkChain<Link> {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(Vec::<Link>::deserialize(deserializer)?))
    }
}

impl<Link> Default for LinkChain<Link> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Link> LinkChain<Link> {
    pub fn new() -> LinkChain<Link> {
        Self(Vec::new())
    }

    pub fn push(&mut self, link: Link) -> Ref<Link> {
        self.0.push(link);
        self.tail()
    }

    pub fn tail(&self) -> Ref<Link> {
        if self.0.is_empty() {
            Ref::Identity
        } else {
            Ref::new(self.0.len() - 1)
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum Ref<Element> {
    Identity,
    Ref(usize, PhantomData<Element>),
}

impl<Element> Ref<Element> {
    pub(crate) const fn new(index: usize) -> Self {
        Self::Ref(index, PhantomData)
    }

    pub(crate) fn index(&self) -> Option<usize> {
        match self {
            Ref::Identity => None,
            Ref::Ref(index, _) => Some(*index),
        }
    }

    pub(crate) fn transmute<Other>(self) -> Ref<Other> {
        match self {
            Ref::Identity => Ref::Identity,
            Ref::Ref(index, _) => Ref::Ref(index, PhantomData),
        }
    }
}

pub type ChainElement<Element> = (Ref<Element>, Element);

/*
impl<Element: Clone> Clone for ChainElement<Element> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent,
            element: self.element.clone(),
        }
    }
}

impl<Element: PartialEq> PartialEq for ChainElement<Element> {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.element == other.element
    }
}

impl<Element: Serialize> Serialize for ChainElement<Element> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize(self.parent)?;
        serializer.serialize(self.element)
    }
}

impl<Element> ChainElement<Element> {
    pub fn new(parent: ElementRef<Element>, element: Element) -> Self {
        Self { parent, element }
    }
}
*/

pub trait Identity {
    const IDENTITY: Self;
}

impl<Link> Chain<Link> for LinkChain<Link> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn resolve(&mut self, r: Ref<Link>) -> Option<&Link> {
        Some(&self.0[r.index()?])
    }
}
