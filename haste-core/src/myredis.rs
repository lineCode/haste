use crate::chunk::{Chunks, ROLE_MASTER, ROLE_SLAVE};

use failure::Error;
use redis::{self, Client, FromRedisValue, Connection};

use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::u64;

pub struct MyRedis {
    nodes: HashMap<String, Node>,
}

impl<'a> From<&'a Chunks> for MyRedis {
    fn from(_chunks: &'a Chunks) -> MyRedis {
        unimplemented!()
    }
}

impl MyRedis {
    pub fn execute<T, C>(&mut self, to: &str, cmd: C) -> Result<T, Error>
    where
        T: FromRedisValue,
        C: Borrow<str>,
    {
        let node = self
            .nodes
            .entry(to.to_string())
            .or_insert_with(|| Node::open(to).expect("fail to open conenction"));
        node.execute(cmd)
    }

    pub fn execute_with<T, C>(&mut self, host: &str, port: usize, cmd: C) -> Result<T, Error>
    where
        T: FromRedisValue,
        C: Borrow<str>,
    {
        self.execute(&*format!("{}:{}", host, port), cmd)
    }

    pub fn execute_all<T, C>(&mut self, cmd: C) -> Result<Vec<T>, Error>
    where
        T: FromRedisValue,
        C: Borrow<str>,
    {
        let mut rslt = Vec::with_capacity(self.nodes.len());
        let hosts: Vec<String> = self.nodes.keys().map(|x| x.to_string()).collect();
        for addr in hosts {
            let value = self.execute(&addr, cmd.borrow())?;
            rslt.push(value);
        }
        Ok(rslt)
    }

    pub fn is_consistent(&mut self) -> Result<bool, Error> {
        let mut latest = u64::MAX;
        for (_addr, node) in self.nodes.iter_mut() {
            let mut hasher = DefaultHasher::default();
            let content: String = node.execute("CLUSTER NODES")?;
            for slot in parse_slots(&content) {
                if slot == "" {
                    return Ok(false);
                }
                hasher.write(slot.as_bytes());
            }
            let value = hasher.finish();
            if latest == u64::MAX {
                latest = value;
            }
            if latest != value {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn bumpepoch(&mut self) -> Result<(), Error> {
        for (_, node) in self.nodes.iter_mut() {
            if node.role == ROLE_MASTER {
                let _: () = node.execute("CLUSTER BUMPEPOCH")?;
            }
        }
        Ok(())
    }

    pub fn balance(&mut self) -> Result<(), Error> {
        for (_, node) in self.nodes.iter_mut() {
            node.try_balance()?;
        }
        Ok(())
    }

    pub fn is_balanced(&mut self) -> Result<bool, Error> {
        for (_, node) in self.nodes.iter_mut() {
            if !node.check_role()? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

struct Node {
    client: Client,
    role: String,
}

impl Node {
    fn try_balance(&mut self) -> Result<(), Error> {
        if self.role == ROLE_MASTER {
            return Ok(());
        }
        let conn = self.client.get_connection()?;
        if self.check_role_with_conn(&conn)? {
            return Ok(());
        }
        let _: () = redis::cmd("CLUSTER").arg("FAILOVER").query(&conn)?;
        Ok(())
    }

    fn check_role_with_conn(&mut self, conn: &Connection) -> Result<bool, Error>{
        let info: String = redis::cmd("INFO").arg("REPLICATION").query(conn)?;
        for line in info.split('\n') {
            if line.contains("role") && line.contains(&self.role) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn check_role(&mut self) -> Result<bool, Error> {
        if self.role == "" {
            return Ok(true);
        }
        let conn = self.client.get_connection()?;
        self.check_role_with_conn(&conn)
    }

    fn open(addr: &str) -> Result<Node, Error> {
        let client = Client::open(addr)?;
        Ok(Node {
            role: ROLE_MASTER.to_string(),
            client,
        })
    }

    pub fn execute<T, C>(&mut self, cmd: C) -> Result<T, Error>
    where
        T: FromRedisValue,
        C: Borrow<str>,
    {
        let conn = self.client.get_connection()?;
        let mut cmds = cmd.borrow().split(' ');
        let cmd = cmds.next().expect("must get commands");
        let mut command = redis::cmd(cmd);
        for arg in cmds {
            command.arg(arg);
        }

        let value = command.query(&conn)?;
        Ok(value)
    }
}

fn parse_slots(src: &str) -> Vec<String> {
    let mut slots = Vec::with_capacity(16384);
    slots.resize(16384, "".to_string());

    for line in src.split('\n') {
        if line.len() == 0 {
            continue;
        }
        if line.contains(ROLE_SLAVE) {
            continue;
        }
        if line.contains("fail") {
            continue;
        }

        let items: Vec<_> = line.split(' ').collect();
        let addr = items[1].split('@').next().unwrap().to_string();

        for item in &items[8..] {
            if item.contains("-<-") {
                continue;
            }

            if item.contains("->-") {
                let first = item
                    .trim_matches('[')
                    .split("->-")
                    .next()
                    .expect("migrating slot must contains first");
                let mslot = first.parse::<usize>().unwrap();
                slots[mslot] = addr.clone();
                continue;
            }

            let mut iter = item.split('-');
            let begin = iter.next().unwrap().parse::<usize>().unwrap();
            let mut end = begin;
            if let Some(ival) = iter.next() {
                end = ival.parse::<usize>().unwrap();
            }
            for s in begin..=end {
                slots[s] = addr.clone();
            }
        }
    }
    slots
}
