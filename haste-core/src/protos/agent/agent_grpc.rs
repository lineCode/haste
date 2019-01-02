// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_AGENT_DEPLOY: ::grpcio::Method<super::agent::CacheInfo, super::agent::CacheState> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/agent.Agent/Deploy",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_AGENT_DO_ACTION: ::grpcio::Method<super::agent::Action, super::agent::CacheState> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/agent.Agent/DoAction",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_AGENT_GET_PORTS: ::grpcio::Method<super::agent::PortAcquire, super::agent::Ports> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/agent.Agent/GetPorts",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct AgentClient {
    client: ::grpcio::Client,
}

impl AgentClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        AgentClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn deploy_opt(&self, req: &super::agent::CacheInfo, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::agent::CacheState> {
        self.client.unary_call(&METHOD_AGENT_DEPLOY, req, opt)
    }

    pub fn deploy(&self, req: &super::agent::CacheInfo) -> ::grpcio::Result<super::agent::CacheState> {
        self.deploy_opt(req, ::grpcio::CallOption::default())
    }

    pub fn deploy_async_opt(&self, req: &super::agent::CacheInfo, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::agent::CacheState>> {
        self.client.unary_call_async(&METHOD_AGENT_DEPLOY, req, opt)
    }

    pub fn deploy_async(&self, req: &super::agent::CacheInfo) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::agent::CacheState>> {
        self.deploy_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn do_action_opt(&self, req: &super::agent::Action, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::agent::CacheState> {
        self.client.unary_call(&METHOD_AGENT_DO_ACTION, req, opt)
    }

    pub fn do_action(&self, req: &super::agent::Action) -> ::grpcio::Result<super::agent::CacheState> {
        self.do_action_opt(req, ::grpcio::CallOption::default())
    }

    pub fn do_action_async_opt(&self, req: &super::agent::Action, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::agent::CacheState>> {
        self.client.unary_call_async(&METHOD_AGENT_DO_ACTION, req, opt)
    }

    pub fn do_action_async(&self, req: &super::agent::Action) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::agent::CacheState>> {
        self.do_action_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_ports_opt(&self, req: &super::agent::PortAcquire, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::agent::Ports> {
        self.client.unary_call(&METHOD_AGENT_GET_PORTS, req, opt)
    }

    pub fn get_ports(&self, req: &super::agent::PortAcquire) -> ::grpcio::Result<super::agent::Ports> {
        self.get_ports_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_ports_async_opt(&self, req: &super::agent::PortAcquire, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::agent::Ports>> {
        self.client.unary_call_async(&METHOD_AGENT_GET_PORTS, req, opt)
    }

    pub fn get_ports_async(&self, req: &super::agent::PortAcquire) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::agent::Ports>> {
        self.get_ports_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Agent {
    fn deploy(&mut self, ctx: ::grpcio::RpcContext, req: super::agent::CacheInfo, sink: ::grpcio::UnarySink<super::agent::CacheState>);
    fn do_action(&mut self, ctx: ::grpcio::RpcContext, req: super::agent::Action, sink: ::grpcio::UnarySink<super::agent::CacheState>);
    fn get_ports(&mut self, ctx: ::grpcio::RpcContext, req: super::agent::PortAcquire, sink: ::grpcio::UnarySink<super::agent::Ports>);
}

pub fn create_agent<S: Agent + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_AGENT_DEPLOY, move |ctx, req, resp| {
        instance.deploy(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_AGENT_DO_ACTION, move |ctx, req, resp| {
        instance.do_action(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_AGENT_GET_PORTS, move |ctx, req, resp| {
        instance.get_ports(ctx, req, resp)
    });
    builder.build()
}
