use etcd::kv::{self, GetOptions, KeyValueInfo};
use etcd::{Client, Response};
use failure::Error;
use futures::Future;
use hyper::client::HttpConnector;
use log::debug;
use tokio::runtime::Runtime;

pub struct MyEtcd {
    client: Client<HttpConnector>,
}

impl MyEtcd {
    // http://etcd.example.com:2379
    pub fn open(addr: &str) -> Result<MyEtcd, Error> {
        let client = Client::new(&[addr], None)?;
        Ok(MyEtcd { client })
    }

    pub fn set(&self, key: &str, val: &str) -> Result<Response<KeyValueInfo>, Error> {
        // let (tx, rx) = channel();
        let work = kv::set(&self.client, key, val, None).and_then(|response| {
            debug!("set response as {:?}", response);
            Ok(response)
        });
        let value = Runtime::new().unwrap().block_on(work).unwrap();
        Ok(value)
    }

    pub fn setnx(&self, key: &str, val: &str, ttl: u64) -> Result<Response<KeyValueInfo>, Error> {
        // let (tx, rx) = channel();
        let work = kv::set(&self.client, key, val, Some(ttl)).and_then(|response| {
            debug!("set response as {:?}", response);
            Ok(response)
        });
        let value = Runtime::new().unwrap().block_on(work).unwrap();
        Ok(value)
    }

    pub fn get(&self, key: &str) -> Result<Response<KeyValueInfo>, Error> {
        let work = kv::get(&self.client, key, GetOptions::default()).and_then(|response| {
            debug!("get response as {:?}", response);
            Ok(response)
        });
        let value = Runtime::new().unwrap().block_on(work).unwrap();
        Ok(value)
    }

    pub fn delete(&self, key: &str) -> Result<Response<KeyValueInfo>, Error> {
        let work = kv::delete(&self.client, key, false).and_then(|response| {
            debug!("get response as {:?}", response);
            Ok(response)
        });
        let value = Runtime::new().unwrap().block_on(work).unwrap();
        Ok(value)
    }
}
