mod implementors;
pub mod providers;

use std::{fmt::Debug, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use implementors::KeyvalueImplementor;
use serde::{Deserialize, Serialize};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
// #[cfg(feature = "wasmtime")]
// use keyvalue::*;
use slight_common::{impl_resource, BasicState};
use slight_file::capability_store::CapabilityStore;
// use slight_file::resource::KeyvalueResource::*;
use slight_file::Resource;
// #[cfg(feature = "wasmtime")]
// wit_bindgen_wasmtime::export!({paths: ["../../wit/keyvalue.wit"], async: *});
// #[cfg(feature = "wasmtime")]
// wit_error_rs::impl_error!(keyvalue::KeyvalueError);
// #[cfg(feature = "wasmtime")]
// wit_error_rs::impl_from!(anyhow::Error, keyvalue::KeyvalueError::UnexpectedError);

// #[cfg(feature = "wasmedge")]
invoke_witc::wit_runtime!(export(wasmedge_keyvalue = "wit/keyvalue.wit"));

/// The `Keyvalue` structure is what will implement the `keyvalue::Keyvalue` trait
/// coming from the generated code of off `keyvalue.wit`.
///
/// It holds:
///     - a `keyvalue_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `config_type`
///     and the `config_toml_file_path`).
#[cfg(feature = "wasmtime")]
#[derive(Clone, Default)]
pub struct Keyvalue {
    implementor: Resource,
    capability_store: CapabilityStore<BasicState>,
}

#[cfg(feature = "wasmtime")]
impl Keyvalue {
    pub fn new(implementor: Resource, keyvalue_store: CapabilityStore<BasicState>) -> Self {
        Self {
            implementor,
            capability_store: keyvalue_store,
        }
    }
}

/// This is the type of the associated type coming from the `keyvalue::Keyvalue` trait
/// implementation.
///
/// It holds:
///     - a `keyvalue_implementor` (i.e., a variant `KeyvalueImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `keyvalye::Keyvalue` cannot leak
/// a private type.
#[cfg(feature = "wasmtime")]
#[derive(Clone, Debug)]
pub struct KeyvalueInner {
    keyvalue_implementor: Arc<dyn KeyvalueImplementor + Send + Sync>,
}

#[cfg(feature = "wasmtime")]
impl KeyvalueInner {
    async fn new(
        keyvalue_implementor: KeyvalueImplementors,
        slight_state: &BasicState,
        name: &str,
    ) -> Self {
        Self {
            keyvalue_implementor: match keyvalue_implementor {
                #[cfg(feature = "filesystem")]
                KeyvalueImplementors::Filesystem => {
                    Arc::new(filesystem::FilesystemImplementor::new(slight_state, name).await)
                }
                #[cfg(feature = "azblob")]
                KeyvalueImplementors::AzBlob => {
                    Arc::new(azblob::AzBlobImplementor::new(slight_state, name).await)
                }
                #[cfg(feature = "awsdynamodb")]
                KeyvalueImplementors::AwsDynamoDb => {
                    Arc::new(awsdynamodb::AwsDynamoDbImplementor::new(slight_state, name).await)
                }
                #[cfg(feature = "redis")]
                KeyvalueImplementors::Redis => {
                    Arc::new(redis::RedisImplementor::new(slight_state, name).await)
                }
            },
        }
    }
}

/// This defines the available implementor implementations for the `Keyvalue` interface.
///
/// As per its' usage in `KeyvalueInner`, it must `derive` `Debug`, and `Clone`.
#[cfg(feature = "wasmtime")]
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum KeyvalueImplementors {
    #[cfg(feature = "filesystem")]
    Filesystem,
    #[cfg(feature = "azblob")]
    AzBlob,
    #[cfg(feature = "awsdynamodb")]
    AwsDynamoDb,
    #[cfg(feature = "redis")]
    Redis,
}

#[cfg(feature = "wasmtime")]
impl From<Resource> for KeyvalueImplementors {
    fn from(s: Resource) -> Self {
        match s {
            #[cfg(feature = "filesystem")]
            Resource::Keyvalue(Filesystem) | Resource::Keyvalue(V1Filesystem) => Self::Filesystem,
            #[cfg(feature = "azblob")]
            Resource::Keyvalue(Azblob) | Resource::Keyvalue(V1Azblob) => Self::AzBlob,
            #[cfg(feature = "awsdynamodb")]
            Resource::Keyvalue(AwsDynamoDb) | Resource::Keyvalue(V1AwsDynamoDb) => {
                Self::AwsDynamoDb
            }
            #[cfg(feature = "redis")]
            Resource::Keyvalue(Redis) | Resource::Keyvalue(V1Redis) => Self::Redis,
            p => panic!(
                "failed to match provided name (i.e., '{p}') to any known host implementations"
            ),
        }
    }
}

// This implements the `CapabilityBuilder`, and `Capability` trait
// for our `Keyvalue` `struct`, and `CapabilityIndexTable` for our `keyvalue::KeyvalueTables` object.
//
// The `CapabilityBuilder` trait provides two functions:
// - `add_to_linker`, and
// - `build_data`.
//
// The `Capability` and `CapabilityIndexTable` traits are empty traits that allow
// grouping of resources through `dyn Capability`, and `dyn CapabilityIndexTable`.
#[cfg(feature = "wasmtime")]
impl_resource!(
    Keyvalue,
    keyvalue::KeyvalueTables<Keyvalue>,
    keyvalue::add_to_linker,
    "keyvalue".to_string()
);

/// This is the implementation for the generated `keyvalue::Keyvalue` trait from the `keyvalue.wit` file.
#[cfg(feature = "wasmtime")]
#[async_trait]
impl keyvalue::Keyvalue for Keyvalue {
    type Keyvalue = KeyvalueInner;

    async fn keyvalue_open(&mut self, name: &str) -> Result<Self::Keyvalue, KeyvalueError> {
        // populate our inner keyvalue object w/ the state received from `slight`
        // (i.e., what type of keyvalue implementor we are using), and the assigned
        // name of the object.
        let s = self.implementor.to_string();
        let state = if let Some(r) = self.capability_store.get(name, "keyvalue") {
            r.clone()
        } else if let Some(r) = self.capability_store.get(&s, "keyvalue") {
            r.clone()
        } else {
            panic!(
                "could not find capability under name '{}' for implementor '{}'",
                name, &s
            );
        };

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let inner = Self::Keyvalue::new(state.implementor.into(), &state, name).await;

        Ok(inner)
    }

    async fn keyvalue_get(
        &mut self,
        self_: &Self::Keyvalue,
        key: &str,
    ) -> Result<Vec<u8>, KeyvalueError> {
        Ok(self_.keyvalue_implementor.get(key).await?)
    }

    async fn keyvalue_set(
        &mut self,
        self_: &Self::Keyvalue,
        key: &str,
        value: &[u8],
    ) -> Result<(), KeyvalueError> {
        self_.keyvalue_implementor.set(key, value).await?;
        Ok(())
    }

    async fn keyvalue_keys(
        &mut self,
        self_: &Self::Keyvalue,
    ) -> Result<Vec<String>, KeyvalueError> {
        Ok(self_.keyvalue_implementor.keys().await?)
    }

    async fn keyvalue_delete(
        &mut self,
        self_: &Self::Keyvalue,
        key: &str,
    ) -> Result<(), KeyvalueError> {
        self_.keyvalue_implementor.delete(key).await?;
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct Keyvalue {
    implementor: Resource,
    capability_store: CapabilityStore<BasicState>,
}

fn keyvalue_open(name: String) -> Result<keyvalue, keyvalue_error> {
    println!("new store `{}`", name);
    Ok(1)
}
fn keyvalue_set(handle: keyvalue, key: String, value: Vec<u8>) -> Result<(), keyvalue_error> {
    println!("insert `{}`", key);
    Ok(())
}
fn keyvalue_get(handle: keyvalue, key: String) -> Result<Vec<u8>, keyvalue_error> {
    println!("get `{}`", key);
    Ok(vec![1])
}
fn keyvalue_keys(handle: keyvalue) -> Result<Vec<String>, keyvalue_error> {
    println!("get keys");
    Ok(vec![String::from("key1")])
}
fn keyvalue_delete(handle: keyvalue, key: String) -> Result<(), keyvalue_error> {
    println!("remove `{}`", key);
    Ok(())
}

impl slight_common::WasmedgeLinkable for Keyvalue {
    fn add_to_linker(vm: wasmedge_sdk::Vm) -> anyhow::Result<wasmedge_sdk::Vm> {
        let r = vm.register_import_module(wasmedge_keyvalue::wit_import_object()?)?;
        Ok(r)
    }
}
