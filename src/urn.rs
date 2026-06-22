use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize};

use crate::resource::ResourceKind;

fn parse_kind(s: &str) -> Result<ResourceKind, Error> {
    ResourceKind::from_str(s).map_err(|_| Error::UnknownKind(s.to_string()))
}

#[inline]
fn fmt_urn(
    f: &mut std::fmt::Formatter<'_>,
    pod: Option<&str>,
    kind: ResourceKind,
    namespace: &str,
    name: &str,
) -> std::fmt::Result {
    f.write_str("urn:brain:")?;

    if let Some(pod) = pod {
        f.write_str("pod:")?;
        f.write_str(pod)?;
        f.write_char(':')?;
    }

    f.write_str(&kind.to_string().to_lowercase())?;
    f.write_char(':')?;
    f.write_str(namespace)?;
    f.write_char(':')?;
    f.write_str(name)
}

/// Errors that can occur when parsing a URN string.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid URN format")]
    Invalid,
    #[error("pod not present in URN")]
    MissingPod,
    #[error("unknown resource kind: {0}")]
    UnknownKind(String),
}

/// Borrowed view of a parsed URN.
pub struct UrnRef<'a> {
    pub pod: Option<&'a str>,
    pub kind: ResourceKind,
    pub namespace: &'a str,
    pub name: &'a str,
}

impl<'a> UrnRef<'a> {
    pub fn parse(s: &'a str) -> Result<Self, Error> {
        if let Some(suffix) = s.strip_prefix("urn:brain:") {
            if let Some(suffix) = suffix.strip_prefix("pod:") {
                let components = suffix.split(':').collect::<Vec<_>>();

                if components.len() == 4 {
                    return Ok(Self {
                        pod: Some(components[0]),
                        kind: parse_kind(components[1])?,
                        namespace: components[2],
                        name: components[3],
                    });
                }
            } else {
                let components = suffix.split(':').collect::<Vec<_>>();

                if components.len() == 3 {
                    return Ok(Self {
                        pod: None,
                        kind: parse_kind(components[0])?,
                        namespace: components[1],
                        name: components[2],
                    });
                }
            }
        }

        Err(Error::Invalid)
    }
}

impl Display for UrnRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_urn(f, self.pod, self.kind, self.namespace, self.name)
    }
}

/// Owned URN that may or may not reference a specific pod.
#[derive(Debug, Clone)]
pub struct Urn {
    pub pod: Option<String>,
    pub kind: ResourceKind,
    pub namespace: String,
    pub name: String,
}

impl From<UrnRef<'_>> for Urn {
    fn from(urn: UrnRef<'_>) -> Self {
        Self {
            pod: urn.pod.map(str::to_string),
            kind: urn.kind,
            namespace: urn.namespace.to_string(),
            name: urn.name.to_string(),
        }
    }
}

impl Display for Urn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_urn(
            f,
            self.pod.as_deref(),
            self.kind,
            &self.namespace,
            &self.name,
        )
    }
}

impl Serialize for Urn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for Urn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let urn = UrnRef::parse(&string).map_err(serde::de::Error::custom)?;
        Ok(Urn::from(urn))
    }
}

/// A borrowed view of a URN that is **guaranteed** to have a pod segment.
pub struct PodUrnRef<'a> {
    pub pod: &'a str,
    pub kind: ResourceKind,
    pub namespace: &'a str,
    pub name: &'a str,
}

impl<'a> PodUrnRef<'a> {
    pub fn parse(s: &'a str) -> Result<Self, Error> {
        let urn = UrnRef::parse(s)?;
        Ok(Self {
            pod: urn.pod.ok_or(Error::MissingPod)?,
            kind: urn.kind,
            namespace: urn.namespace,
            name: urn.name,
        })
    }
}

impl Display for PodUrnRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_urn(f, Some(self.pod), self.kind, self.namespace, self.name)
    }
}

/// An owned, pod-scoped URN.
///
/// The owned counterpart of [`PodUrnRef`];
pub struct PodUrn {
    pub pod: String,
    pub kind: ResourceKind,
    pub namespace: String,
    pub name: String,
}

impl From<PodUrnRef<'_>> for PodUrn {
    fn from(urn: PodUrnRef<'_>) -> Self {
        Self {
            pod: urn.pod.to_string(),
            kind: urn.kind,
            namespace: urn.namespace.to_string(),
            name: urn.name.to_string(),
        }
    }
}

impl Display for PodUrn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_urn(f, Some(&self.pod), self.kind, &self.namespace, &self.name)
    }
}
