// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use consensus::{consensus_impl, difficult, dummy::DummyHeader, Consensus};
use futures::channel::oneshot;
use logger::prelude::*;
use std::convert::TryFrom;
use std::sync::Arc;
use std::sync::Mutex;

use crypto::HashValue;
use traits::ChainReader;
use types::{
    block::BlockTemplate, system_events::SystemEvents, transaction::SignedUserTransaction,
};
#[derive(Clone)]
pub struct Miner {
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: Addr<BusActor>,
}

#[derive(Clone)]
pub struct MineCtx {
    header_hash: HashValue,
    block_template: BlockTemplate,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate) -> MineCtx {
        let header_hash = block_template
            .clone()
            .into_block_header(DummyHeader {})
            .id();

        MineCtx {
            header_hash,
            block_template,
        }
    }
}

pub fn mint<C>(
    config: Arc<NodeConfig>,
    txns: Vec<SignedUserTransaction>,
    chain: &dyn ChainReader,
    bus: Addr<BusActor>,
) -> Result<()>
where
    C: Consensus,
{
    let difficulty = difficult::get_next_work_required(chain);
    let block_template = chain.create_block_template(difficulty, txns)?;
    let (_sender, receiver) = oneshot::channel();
    // spawn a async task, maintain a task list, when new task coming, cancel old task.
    let block = C::create_block(config, chain, block_template, receiver)?;
    // fire SystemEvents::MinedBlock.
    //TODO handle result.
    info!("broadcast new block: {:?}.", block.header().id());
    bus.do_send(Broadcast {
        msg: SystemEvents::MinedBlock(block),
    });
    Ok(())
}

impl Miner {
    pub fn new(bus: Addr<BusActor>) -> Miner {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
        }
    }
    pub fn set_mint_job(&mut self, t: MineCtx) {
        let mut state = self.state.lock().unwrap();
        *state = Some(t)
    }

    pub fn get_mint_job(&mut self) -> String {
        let state = self.state.lock().unwrap();
        let x = state.as_ref().unwrap().to_owned();
        format!(
            r#"[0x{:x},0x{:x}]"#,
            x.header_hash, x.block_template.difficult
        )
    }

    pub fn submit(&self, payload: Vec<u8>) {
        // verify payload
        // create block
        let state = self.state.lock().unwrap();
        let block_template = state.as_ref().unwrap().block_template.clone();
        let consensus_header = consensus_impl::ConsensusHeaderImpl::try_from(payload).unwrap();
        let block = block_template.into_block(consensus_header);
        // notify chain mined block
        info!("Miner new block: {:?}", block);
        // fire SystemEvents::MinedBlock.
        info!("Broadcast new block: {:?}.", block.header().id());
        self.bus.do_send(Broadcast {
            msg: SystemEvents::MinedBlock(block),
        });
    }
}