use anyhow::Result;
use wasmedge_sdk::Vm;

/// A trait for WasmedgeLinkable resources
pub trait WasmedgeLinkable {
    /// Link the resource to the runtime
    fn add_to_linker(vm: Vm) -> Result<Vm>;
}
