use async_trait::async_trait;

use wasmedge_sdk::Vm;

/// A trait for builder
#[async_trait]
pub trait WasmedgeBuildable: Clone {
    async fn build(self) -> (Vm, String);
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
