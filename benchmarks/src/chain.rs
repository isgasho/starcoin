// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use criterion::{BatchSize, Bencher};
use parking_lot::RwLock;
use rand::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_config::{temp_path, DataDirPath};
use starcoin_consensus::Consensus;
use starcoin_executor::{create_signed_txn_with_association_account, encode_transfer_script};
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_vm_types::chain_config::{ChainId, ChainNetwork, ConsensusStrategy};
use starcoin_vm_types::transaction::{Script, SignedUserTransaction, TransactionPayload};
use std::ops::Deref;
use std::sync::Arc;
use traits::{ChainReader, ChainWriter};

/// Benchmarking support for chain.
pub struct ChainBencher {
    chain: Arc<RwLock<BlockChain>>,
    block_num: u64,
    account: AccountInfo,
    temp_path: DataDirPath,
}

fn create_transaction(sequence_number: u64, program: Script) -> SignedUserTransaction {
    create_signed_txn_with_association_account(
        TransactionPayload::Script(program),
        sequence_number,
        400_000,
        1,
        ChainNetwork::Test.consensus().now() + sequence_number + 1,
        ChainId::test(),
    )
}

impl ChainBencher {
    pub fn new(num: Option<u64>) -> Self {
        let net = ChainNetwork::Test;
        let temp_path = temp_path();
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_and_db_instance(
                CacheStorage::new(),
                DBStorage::new(temp_path.path().join("starcoindb")),
            ))
            .unwrap(),
        );
        let (startup_info, _) =
            Genesis::init_and_check_storage(net, storage.clone(), temp_path.path())
                .expect("init storage by genesis fail.");

        let chain = BlockChain::new(net, startup_info.master, storage, None)
            .expect("create block chain should success.");
        let miner_account = AccountInfo::random();

        ChainBencher {
            chain: Arc::new(RwLock::new(chain)),
            block_num: match num {
                Some(n) => n,
                None => 100,
            },
            account: miner_account,
            temp_path,
        }
    }

    pub fn execute(&self) {
        let script = encode_transfer_script(
            *self.account.address(),
            self.account.get_auth_key().prefix().to_vec(),
            1,
        );
        for i in 0..self.block_num {
            //let mut txn_vec = Vec::new();
            //txn_vec.push(random_txn(self.count.load(Ordering::Relaxed)));
            let (block_template, _) = self
                .chain
                .read()
                .create_block_template(
                    *self.account.address(),
                    Some(self.account.get_auth_key().prefix().to_vec()),
                    None,
                    vec![create_transaction(i, script.clone())],
                    vec![],
                    None,
                )
                .unwrap();
            let block = ConsensusStrategy::Dummy
                .create_block(self.chain.read().deref(), block_template)
                .unwrap();
            self.chain.write().apply(block).unwrap();
        }
    }

    fn execute_query(&self, times: u64) {
        let max_num = self.chain.read().current_header().number();
        let mut rng = rand::thread_rng();
        for _i in 0..times {
            let number = rng.gen_range(0, max_num);
            assert!(self
                .chain
                .read()
                .get_block_by_number(number)
                .unwrap()
                .is_some());
        }
    }

    pub fn query_bench(&self, b: &mut Bencher, times: u64) {
        b.iter_batched(
            || (self, times),
            |(bench, t)| bench.execute_query(t),
            BatchSize::LargeInput,
        )
    }

    pub fn bench(&self, b: &mut Bencher) {
        b.iter_batched(|| self, |bench| bench.execute(), BatchSize::LargeInput)
    }
}

impl Clone for ChainBencher {
    fn clone(&self) -> Self {
        Self {
            chain: self.chain.clone(),
            block_num: self.block_num,
            account: self.account.clone(),
            temp_path: self.temp_path.clone(),
        }
    }
}
