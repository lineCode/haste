use crate::chunk::{Chunks, ROLE_MASTER};

use failure::Error;
use redis::{self, Client, FromRedisValue};

use std::borrow::Borrow;
use std::collections::HashMap;

pub struct MyRedis {
    nodes: HashMap<String, Node>,
}

impl<'a> From<&'a Chunks> for MyRedis {
    fn from(chunks: &'a Chunks) -> MyRedis {
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
        unimplemented!()
    }

    pub fn bumpepoch(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn balance(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn is_balanced(&mut self) -> Result<bool, Error> {
        unimplemented!()
    }
}

struct Node {
    client: Client,
    role: String,
}

impl Node {
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
