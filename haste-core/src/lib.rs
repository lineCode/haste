//! ## deploy process
//!  * valid the input params in the front end.
//!  * call chunks/dist in the front end
//!  * check if cluster was existing
//!  * mark the name as a registed name
//!  * send into worker
//!  * worker mark and register into the job
//!  * worker send grpc to agent deploy cluster
//!  * agent check or fetch binary with flock
//!  * agent render template file
//!  * agent setup systemd
//!  * agent lock and call systemctl daemon-reaload
//!  * agent startup systemd service
//!  * worker receive last grpc result
//!  * worker spawn monitor check if cluster is fullly done
//!  * worker spawn balance task (if is redis_cluster)
//!  * worker mark the job done
//!
//! ## remove instance process
//!  * check if the process is clean
//!  * check if the process is in low ops
//!  * send into worker
//!  * worker send task to agent
//!  * agent stop systemd service
//!  * agent remove systemd service
//!  * agent flock and call daemon reload
//!  * agent remove config files
//!  * agent report done
//!
//! ## agent restart process
//!  * agent call systemd restart
//!
//! ## agent stop/start process
//!  * agent call systemd stop/start
//!
//! # job abstraction
//!  * job create new dir in etcd as /haste/jobs/{job_id}
//!  * job create audit log dir and append into the clusters
//!  * job create state for this job
//!  * job was registered by worker in /haste/jobs/{job_id}/worker
//!  * job create start/done/latest_update time in dir
//!
//! # error due:
//!  * clean all the meta data if taks is not good done
//!  * when some one is not ready
//!  * trying to send restart commanad when deploy fail.
//!

#[macro_use]
extern crate tera;

use haste_info::say;

pub mod chunk;
pub mod deploy;
pub mod myetcd;
pub mod myredis;
pub mod offer;
mod protos;
pub mod systemd;

pub use self::protos::agent::agent as proto;
pub use self::protos::agent::agent_grpc as proto_grpc;

pub fn run() {
    say();
}

/// server acquire resource -> by using offer
/// agent report Offer by using offer
#[derive(Clone, Debug)]
pub struct Offer {
    host: String,
    cpu: f64,
    memory: f64,
    ports: Vec<u32>,
}
