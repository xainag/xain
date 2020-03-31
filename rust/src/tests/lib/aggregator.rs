use crate::{
    aggregator::service::{Aggregator, ServiceHandle as InnerServiceHandle, ServiceRequests},
    common::client::Credentials,
};
use bytes::Bytes;
use std::{future::Future, pin::Pin};

#[derive(Clone)]
pub struct ServiceHandle(InnerServiceHandle);

pub struct ByteAggregator {
    weights: Vec<u8>,
}

impl ByteAggregator {
    pub fn new() -> ByteAggregator {
        ByteAggregator { weights: vec![] }
    }
}

impl Aggregator for ByteAggregator {
    type Error = ();

    type AddWeightsFut = Pin<Box<dyn Future<Output = Result<(), ()>> + Send>>;
    type AggregateFut = Pin<Box<dyn Future<Output = Result<Bytes, ()>> + Send>>;

    fn add_weights(&mut self, weights: Bytes) -> Self::AddWeightsFut {
        weights.into_iter().for_each(|el| self.weights.push(el));
        Box::pin(async move { Ok(()) })
    }

    fn aggregate(&mut self) -> Self::AggregateFut {
        self.weights.sort();
        let global_weights = Bytes::copy_from_slice(&self.weights[..]);
        Box::pin(async move { Ok(global_weights) })
    }
}

impl ServiceHandle {
    pub fn new() -> (Self, ServiceRequests) {
        let (inner, requests) = InnerServiceHandle::new();
        (Self(inner), requests)
    }

    pub async fn download(&self, credentials: Credentials) -> Option<Bytes> {
        self.0.download(credentials).await
    }

    pub async fn upload(&self, credentials: Credentials, data: Bytes) {
        self.0.upload(credentials, data).await
    }

    pub async fn aggregate(&self) -> Result<(), ()> {
        self.0.aggregate().await
    }

    pub async fn select(&self, credentials: Credentials) -> Result<(), ()> {
        self.0.select(credentials).await
    }
}
