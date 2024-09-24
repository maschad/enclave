use std::collections::HashSet;

use actix::prelude::*;
use sortition::DistanceSortition;

use crate::{CiphernodeAdded, CiphernodeRemoved, EnclaveEvent, EventBus, Subscribe};

#[derive(Message, Clone, Debug, PartialEq, Eq)]
#[rtype(result = "bool")]
pub struct GetHasNode {
    pub seed: u64,
    pub address: String,
    pub size: usize,
}

pub trait SortitionList<T> {
    fn contains(&self, seed: u64, size: usize, address: T) -> bool;
    fn add(&mut self, address: T);
    fn remove(&mut self, address: T);
}

pub struct SortitionModule {
    nodes: HashSet<String>,
}

impl SortitionModule {
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
        }
    }
}

impl Default for SortitionModule {
    fn default() -> Self {
        Self::new()
    }
}

impl SortitionList<String> for SortitionModule {
    fn contains(&self, seed: u64, size: usize, address: String) -> bool {
        DistanceSortition::new(
            seed,
            self.nodes
                .clone()
                .into_iter()
                .map(|b| b.parse().unwrap())
                .collect(),
            size,
        )
        .get_committee()
        .iter()
        .any(|(_, addr)| addr.to_string() == address)
    }

    fn add(&mut self, address: String) {
        self.nodes.insert(address);
    }

    fn remove(&mut self, address: String) {
        self.nodes.remove(&address);
    }
}

pub struct Sortition {
    list: SortitionModule,
}

impl Sortition {
    pub fn new() -> Self {
        Self {
            list: SortitionModule::new(),
        }
    }

    pub fn attach(bus: Addr<EventBus>) -> Addr<Sortition> {
        let addr = Sortition::new().start();
        bus.do_send(Subscribe::new("CiphernodeAdded", addr.clone().into()));
        addr
    }
}

impl Default for Sortition {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for Sortition {
    type Context = Context<Self>;
}

impl Handler<EnclaveEvent> for Sortition {
    type Result = ();
    fn handle(&mut self, msg: EnclaveEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            EnclaveEvent::CiphernodeAdded { data, .. } => ctx.notify(data.clone()),
            EnclaveEvent::CiphernodeRemoved { data, .. } => ctx.notify(data.clone()),
            _ => (),
        }
    }
}

impl Handler<CiphernodeAdded> for Sortition {
    type Result = ();
    fn handle(&mut self, msg: CiphernodeAdded, _ctx: &mut Self::Context) -> Self::Result {
        println!("$$$$$$\nCIPHERNODE ADDED\n$$$$$$\n\n{}", msg.address);
        self.list.add(msg.address);
    }
}

impl Handler<CiphernodeRemoved> for Sortition {
    type Result = ();
    fn handle(&mut self, msg: CiphernodeRemoved, _ctx: &mut Self::Context) -> Self::Result {
        self.list.remove(msg.address);
    }
}

impl Handler<GetHasNode> for Sortition {
    type Result = bool;
    fn handle(&mut self, msg: GetHasNode, _ctx: &mut Self::Context) -> Self::Result {
        self.list.contains(msg.seed, msg.size, msg.address)
    }
}
