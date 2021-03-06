use crate::block_connector::WriteBlockChainService;

use chain::BlockChain;
use config::NodeConfig;
use consensus::Consensus;
use starcoin_account_api::AccountInfo;
use starcoin_genesis::Genesis as StarcoinGenesis;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_txpool_mock_service::MockTxPoolService;
use std::sync::Arc;
use traits::{ChainReader, WriteableChainService};

async fn create_writeable_block_chain(
) -> (WriteBlockChainService<MockTxPoolService>, Arc<NodeConfig>) {
    let node_config = NodeConfig::random_for_test();
    let node_config = Arc::new(node_config);

    let (storage, startup_info, _) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let registry = RegistryService::launch();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    let txpool_service = MockTxPoolService::new();
    (
        WriteBlockChainService::new(
            node_config.clone(),
            startup_info,
            storage,
            txpool_service,
            bus,
            None,
        )
        .unwrap(),
        node_config,
    )
}

fn gen_blocks(
    node_config: Arc<NodeConfig>,
    times: u64,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
) {
    let miner_account = AccountInfo::random();
    let consensus_strategy = node_config.net().consensus();
    if times > 0 {
        for _i in 0..times {
            let block_chain = writeable_block_chain_service.get_master();
            let (block_template, _) = block_chain
                .create_block_template(
                    *miner_account.address(),
                    Some(miner_account.public_key.clone()),
                    None,
                    Vec::new(),
                    vec![],
                    None,
                )
                .unwrap();
            let block = consensus_strategy
                .create_block(block_chain, block_template)
                .unwrap();

            writeable_block_chain_service.try_connect(block).unwrap();
        }
    }
}

#[stest::test]
async fn test_block_chain_apply() {
    let times = 10;
    let (mut writeable_block_chain_service, node_config) = create_writeable_block_chain().await;
    gen_blocks(node_config, times, &mut writeable_block_chain_service);
    assert_eq!(
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number(),
        times
    );
}

fn gen_fork_block_chain(
    fork_number: u64,
    node_config: Arc<NodeConfig>,
    times: u64,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
) {
    let miner_account = AccountInfo::random();
    if let Some(block_header) = writeable_block_chain_service
        .get_master()
        .get_header_by_number(fork_number)
        .unwrap()
    {
        let mut parent_id = block_header.id();
        for _i in 0..times {
            let block_chain = BlockChain::new(
                node_config.clone().net().consensus(),
                parent_id,
                writeable_block_chain_service.get_master().get_storage(),
            )
            .unwrap();
            let (block_template, _) = block_chain
                .create_block_template(
                    *miner_account.address(),
                    Some(miner_account.public_key.clone()),
                    None,
                    Vec::new(),
                    vec![],
                    None,
                )
                .unwrap();
            let block = node_config
                .clone()
                .net()
                .consensus()
                .create_block(&block_chain, block_template)
                .unwrap();
            parent_id = block.id();

            writeable_block_chain_service.try_connect(block).unwrap();
        }
    }
}

#[stest::test]
async fn test_block_chain_forks() {
    let times = 10;
    let (mut writeable_block_chain_service, node_config) = create_writeable_block_chain().await;
    gen_blocks(
        node_config.clone(),
        times,
        &mut writeable_block_chain_service,
    );
    assert_eq!(
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number(),
        times
    );

    gen_fork_block_chain(
        0,
        node_config,
        times / 2,
        &mut writeable_block_chain_service,
    );

    assert_eq!(
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number(),
        times
    );
}

#[stest::test]
async fn test_block_chain_switch_master() {
    let times = 10;
    let (mut writeable_block_chain_service, node_config) = create_writeable_block_chain().await;
    gen_blocks(
        node_config.clone(),
        times,
        &mut writeable_block_chain_service,
    );
    assert_eq!(
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number(),
        times
    );

    gen_fork_block_chain(
        0,
        node_config,
        2 * times,
        &mut writeable_block_chain_service,
    );

    assert_eq!(
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number(),
        2 * times
    );
}
