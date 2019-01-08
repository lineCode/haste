use crate::chunk::{chunk_it, Chunks};
use crate::myredis::MyRedis;
use crate::offer::fetch_offer;
use crate::proto::{CacheInfo, CacheType, File, Instance};
use crate::proto_grpc::{Agent, AgentClient};

use failure::{format_err, Error};
use grpcio::{ChannelBuilder, EnvBuilder};
use log::{error, info, warn};

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

type CacheInfos = HashMap<String, CacheInfo>;

// etcd path
//  /haste/clusters/name/instances/{ip}:{port}/[state,role,slaveof,slots]
//                      /appids/{appids}
//                      /cache_type -> {redis, redis_cluster, memcache}
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
    pub version: String,
    pub tpl_name: String,
    pub cache_type: CacheType,
    pub appids: String,
    pub group: String,
}

pub struct Template {}

impl Template {
    fn render(&self, _chunks: &Chunks, _host: &String, _port: usize) -> Vec<File> {
        unimplemented!()
    }
}

pub struct DeployTask {
    retry: usize,
    param: DeployParm,
    myredis: MyRedis,
}

impl DeployTask {
    pub fn deploy(&mut self) -> Result<(), Error> {
        info!("start to deploy cluster with param {:?}", self.param);

        let chunks = self.create_chunks()?;
        self.myredis.set_chunks(&chunks);

        let template = self.load_template()?;
        let cache_infos = self.chunks_as_cache_infos(&chunks, &template);

        self.retry_deploy(&cache_infos)?;

        thread::sleep(Duration::from_secs(1));

        if let Err(err) = self.check_all_done(&cache_infos) {
            warn!("fail to check all cluster done due {}", err);
            return self.send_clean();
        }

        if let CacheType::RedisCluster = self.param.cache_type {
            self.balance()?;
        }

        self.save_into_etcd(&chunks, &template)?;
        Ok(())
    }

    fn chunks_as_cache_infos(&mut self, chunks: &Chunks, template: &Template) -> CacheInfos {
        let mut inst_map = HashMap::new();
        for inst in chunks.0.iter() {
            let handle = inst_map
                .entry(inst.host.to_string())
                .or_insert_with(|| Vec::new());
            handle.push(inst.clone());
        }
        inst_map
            .into_iter()
            .map(|(host, insts)| {
                let mut info = CacheInfo::new();
                info.set_cache_type(self.param.cache_type);
                info.set_cluster(self.param.name.clone());
                info.set_version(self.param.version.clone());

                let mut instances = Vec::new();
                for i in &insts[..] {
                    let files = template.render(chunks, &i.host, i.port);
                    let mut instance = Instance::new();
                    instance.set_port(i.port as i64);
                    instance.set_files(files.into());
                    instances.push(instance);
                }
                info.set_insts(instances.into());
                (host, info)
            })
            .collect()
    }

    fn retry_deploy(&mut self, cache_infos: &CacheInfos) -> Result<(), Error> {
        for i in 1..=self.retry {
            if let Err(err) = self.send_deploy(cache_infos.clone()) {
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

    fn check_all_done(&mut self, cache_infos: &CacheInfos) -> Result<(), Error> {
        let ths: Vec<_> = cache_infos
            .iter()
            .map(|(host, ci)| {
                let mut ths = Vec::new();
                for inst in ci.get_insts().into_iter() {
                    let host = host.to_string();
                    let port = inst.get_port();
                    let th = thread::spawn(move || check_redis(&format!("{}:{}", host, port)));
                    ths.push(th);
                }
                ths
            })
            .flatten()
            .collect();
        if !ths
            .into_iter()
            .map(|x| x.join().unwrap())
            .inspect(|rslt| {
                if rslt.is_err() {
                    error!("fail to check state due {:?}", rslt);
                }
            })
            .all(|rslt| rslt.is_ok())
        {
            return Err(format_err!("check redis fail finally due"));
        }
        Ok(())
    }

    fn balance(&mut self) -> Result<(), Error> {
        // wait for cluster consistent
        // # if not try bump epoch
        // wait for totally in 3 minutes
        // each wait with 3 second
        let instant = Instant::now();

        // trying to balance by send failover
        loop {
            if instant.elapsed().as_secs() > 60 * 3 {
                return Ok(());
            }

            thread::sleep(Duration::from_secs(3));

            if !self.myredis.is_consistent()? {
                self.myredis.bumpepoch()?;
                continue;
            }

            if self.myredis.is_balanced()? {
                return Ok(());
            }
            self.myredis.balance()?;
        }
    }

    fn send_deploy(&self, cache_infos: CacheInfos) -> Result<(), Error> {
        let mut ths = Vec::new();
        for (host, cache_info) in cache_infos.into_iter() {
            let th = thread::spawn(move || {
                let env = Arc::new(EnvBuilder::new().build());
                let addr = get_grpc_addr(&host).unwrap();
                let ch = ChannelBuilder::new(env).connect(&addr);
                let client = AgentClient::new(ch);
                client.deploy(&cache_info)
            });
            ths.push(th);
        }

        for th in ths {
            let reply = th.join().unwrap();
            reply.expect("fail to send rpc");
        }

        Ok(())
    }

    fn send_clean(&self) -> Result<(), Error> {
        // TODO: impl it latter
        Ok(())
    }

    fn create_chunks(&self) -> Result<Chunks, Error> {
        let offers = fetch_offer();
        let num = (self.param.total_memory / self.param.max_memory + 1) / 2 * 2;
        info!("chunk_it with num {}", num);
        let chunks = chunk_it(num, self.param.cpu_percent, self.param.max_memory, &offers)?;

        Ok(chunks)
    }

    fn save_into_etcd(&mut self, _chunks: &Chunks, _tempate: &Template) -> Result<(), Error> {
        unimplemented!()
    }

    fn load_template(&self) -> Result<Template, Error> {
        unimplemented!()
    }
}

pub struct Dist {}

fn get_grpc_addr(host: &str) -> Option<String> {
    unimplemented!()
}

fn check_redis(addr: &str) -> Result<(), Error> {
    let client = redis::Client::open(addr)?;
    let conn = client.get_connection()?;
    let _: () = redis::cmd("PING").query(&conn)?;
    Ok(())
}
