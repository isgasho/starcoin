address 0x1 {

module TransactionTimeout {
  use 0x1::Signer;
  use 0x1::CoreAddresses;
  use 0x1::Timestamp;
  use 0x1::ErrorCode;
  use 0x1::Block;

  spec module {
      pragma verify;
      pragma aborts_if_is_strict;
  }

  const ONE_DAY :u64 = 86400;
  resource struct TTL {
    // Only transactions with timestamp in between block time and block time + duration would be accepted.
    duration_seconds: u64,
  }

  public fun initialize(account: &signer) {
    // Only callable by the Genesis address
    assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
    // Currently set to 1day.
    //TODO set by onchain config.
    move_to(account, TTL {duration_seconds: ONE_DAY});
  }
  spec fun initialize {
    aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
    aborts_if exists<TTL>(Signer::spec_address_of(account));
    ensures global<TTL>(Signer::spec_address_of(account)).duration_seconds == ONE_DAY;
  }

  public fun set_timeout(account: &signer, new_duration: u64) acquires TTL {
    // Only callable by the Genesis address
    assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

    let timeout = borrow_global_mut<TTL>(CoreAddresses::GENESIS_ADDRESS());
    timeout.duration_seconds = new_duration;
  }
  spec fun set_timeout {
    aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
    aborts_if !exists<TTL>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    ensures global<TTL>(CoreAddresses::SPEC_GENESIS_ADDRESS()).duration_seconds == new_duration;
  }

  public fun is_valid_transaction_timestamp(txn_timestamp: u64): bool acquires TTL {
    let current_block_time = Timestamp::now_seconds();
    let block_number = Block::get_current_block_number();
    // before first block, just require txn_timestamp > genesis timestamp.
    if (block_number == 0) {
      return txn_timestamp > current_block_time
    };
    let timeout = borrow_global<TTL>(CoreAddresses::GENESIS_ADDRESS()).duration_seconds;
    let max_txn_time = current_block_time + timeout;
    current_block_time < txn_timestamp && txn_timestamp < max_txn_time
  }
  spec fun is_valid_transaction_timestamp {
    pragma verify = false;
    aborts_if !exists<Timestamp::CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    aborts_if !exists<Block::BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    // Todo: function does not abort under this condition?
    aborts_if !exists<TTL>(CoreAddresses::SPEC_GENESIS_ADDRESS());
  }
}
}
