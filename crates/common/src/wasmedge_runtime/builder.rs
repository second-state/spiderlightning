use async_trait::async_trait;

use wasmedge_sdk::VmBuilder;

use crate::Ctx;

/// A trait for builder
#[async_trait]
pub trait WasmedgeBuildable: Clone {
    type Ctx: Ctx + Send + Sync;

    async fn build(self) -> VmBuilder;
}

#[derive(Clone)]
pub struct Builder<T: WasmedgeBuildable> {
    inner: T,
}

impl<T: WasmedgeBuildable> Builder<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn owned_inner(self) -> T {
        self.inner
    }
}
