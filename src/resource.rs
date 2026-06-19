use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumString};

use crate::urn::Urn;

/// Common metadata attached to every resource.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The resource's name.
    pub name: String,
    /// The Kubernetes-style namespace this resource belongs to.
    pub namespace: String,
}

pub mod app {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Runtime {
        pub uid: Option<u32>,
        pub gid: Option<u32>,
        pub fs_group: Option<u32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SecretRef {
        pub name: String,
        pub key: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Env {
        Secret { name: String, secret: SecretRef },
        Var { name: String, value: String },
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum InitLifecycle {
        Image(String),
        Full { image: String, cmd: Vec<String> },
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Mount {
        Disk {
            path: PathBuf,
            disk: Urn,
        },
        File {
            path: PathBuf,
            file: String,
            config: String,
        },
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Lifecycle {
        pub init: InitLifecycle,
    }

    /// Selects which build artifact to deploy, or indicates that none has been configured yet.
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "status", rename_all = "camelCase")]
    pub enum ArtifactSelector {
        /// No artifact has been configured; the app is waiting for one.
        Pending,
        /// An artifact has been fully configured and is ready to use.
        #[serde(rename_all = "camelCase")]
        Configured {
            /// Human-readable artifact name.
            name: String,
            /// GitHub repository (e.g. `"org/repo"`) that produced this artifact.
            github_repo: String,
            /// Branch, tag, or commit SHA the artifact was built from.
            ref_name: String,
        },
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Instance {
        #[serde(rename = ".25x")]
        _025x,
        #[serde(rename = ".5x")]
        _05x,
        #[serde(rename = "1x")]
        _1x,
        #[serde(rename = "2x")]
        _2x,
        #[serde(rename = "4x")]
        _4x,
        #[serde(rename = "8x")]
        _8x,
    }

    /// Desired state of an [`App`].
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Spec {
        /// OCI image reference to run (e.g. `"registry.example.com/myapp:latest"`).
        pub image: String,
        /// Optional artifact to watch for deployment
        /// When `None` the app runs without a linked artifact.
        pub artifact_selector: Option<ArtifactSelector>,
        pub env: Vec<Env>,
        pub lifecycle: Option<Lifecycle>,
        pub mounts: Vec<Mount>,
        pub instance: Instance,
        pub replicas: u32,
    }

    /// App resource
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct App {
        /// Name and namespace of this app.
        pub metadata: Metadata,
        /// Desired configuration for this app.
        pub spec: Spec,
    }
}

pub mod disk {
    use super::*;

    /// Observed status of a persistent Disk resource.
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Status {
        /// Provisioning phase (e.g. `"Pending"`, `"Bound"`, `"Released"`).
        pub phase: String,
        /// `true` when the disk has been successfully bound to a claim.
        pub bound: bool,
        /// `true` when the disk is ready for I/O.
        pub ready: bool,
    }
}

pub mod route {
    use super::*;

    /// Observed status of a Route resource.
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Status {
        /// `true` when the route is accepting traffic.
        pub ready: bool,
    }
}

pub use app::App;

/// Every type of resource the platform can manage.
#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, EnumString, EnumIs, Display,
)]
pub enum ResourceKind {
    /// A long-running application workload.
    App,
    /// An HTTPS route.
    Route,
    /// A persistent block-storage volume.
    Disk,
    /// A generic key-value configuration resource.
    Config,
    /// A managed PostgreSQL database instance.
    Postgres,
    /// A managed MariaDB database instance.
    MariaDB,
    /// A managed Valkey (Redis-compatible) cache instance.
    Valkey,
}

impl ResourceKind {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::App,
            Self::Route,
            Self::Disk,
            Self::Config,
            Self::Postgres,
            Self::MariaDB,
            Self::Valkey,
        ]
        .into_iter()
    }

    pub fn from_lowercase_str(s: &str) -> Option<Self> {
        match s {
            "app" => Some(Self::App),
            "route" => Some(Self::Route),
            "disk" => Some(Self::Disk),
            "config" => Some(Self::Config),
            "postgres" => Some(Self::Postgres),
            "mariadb" => Some(Self::MariaDB),
            "valkey" => Some(Self::Valkey),
            _ => None,
        }
    }

}
