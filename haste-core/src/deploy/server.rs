use crate::proto::CacheType;
use crate::offer::acquire_offer;

use failure::{format_err, Error};
use log::{info, warn};

use std::thread;
use std::time::Duration;

// etcd path
//  /haste/clusters/name/instances/{ip}:{port}/state
//                      /appids/{appids}
//                      /cache_type: {redis, redis_cluster, memcache}
//                      /chunks/{ip}:{port}/[role,slaveof,slots]
//                      /audit/{task_id}/[checkpoint, state]
//                      /feport
//                      /config/[dial_timeout,fetch_interval]
//  /haste/appids/{appid}/{cluster_name}/[config]/[dial_timeout,fetch_interval]
//  /haste/templates/cache_type/name/
#[derive(Clone, Debug)]
pub struct DeployParm {
    pub name: String,

    // that means 1 is 1%, 120 is 120%(1.2)
    // in redis, that always be 1
    // in memcache, that must be set.
    pub cpu_percent: usize,
    pub max_memory: usize,
    pub total_memory: usize,
    pub version: usize,
    pub tpl_name: String,
    pub cache_type: CacheType,
    pub appids: String,
    pub group: String,
}

pub struct Template {}

pub struct DeployTask {
    retry: usize,
    param: DeployParm,
}

impl DeployTask {
    pub fn deploy(&mut self) -> Result<(), Error> {
        info!("start to deploy cluster with param {:?}", self.param);
        let chunks = self.create_chunks()?;
        let template = self.load_template()?;
        self.save_into_etcd(chunks, template)?;
        self.retry_deploy()?;
        if let Err(err) = self.check_all_done() {
            warn!("fail to check all cluster done due {}", err);
            return self.send_clean();
        }

        if let CacheType::RedisCluster = self.param.cache_type {
            self.balance()?;
        }
        Ok(())
    }

    fn retry_deploy(&mut self) -> Result<(), Error> {
        for i in 1..=self.retry {
            if let Err(err) = self.send_deploy() {
                warn!(
                    "fail to create cluster {:?} in retry {} due to {}",
                    self.param, i, err
                );

                if let Err(err) = self.send_clean() {
                    warn!(
                        "fail to clean cluster {:?} in retry {} due to {}",
                        self.param, i, err
                    );
                }
                thread::sleep(Duration::from_secs(1));
                continue;
            }

            return Ok(());
        }

        Err(format_err!("fail to create cluster at lest"))
    }

    fn save_into_etcd(&mut self, _chunks: Chunks, _tempate: Template) -> Result<(), Error> {
        unimplemented!()
    }

    fn check_all_done(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn balance(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    fn send_deploy(&self) -> Result<(), Error> {
        unimplemented!();
    }

    fn send_clean(&self) -> Result<(), Error> {
        unimplemented!();
    }

    fn load_template(&self) -> Result<Template, Error> {
        unimplemented!()
    }

    fn create_chunks(&self) -> Result<Chunks, Error> {
        let offers = acquire_offer();

        Err(format_err!("fail to make chunks"))
    }
}

pub struct Chunks {}

pub struct Dist {}
