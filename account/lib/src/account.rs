use crate::account_storage::AccountStorage;
use anyhow::{format_err, Result};
use starcoin_account_api::error::AccountError;
use starcoin_account_api::{AccountInfo, AccountResult};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::PrivateKey;
use starcoin_storage::storage::StorageInstance;
use starcoin_types::account_address;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

pub struct Account {
    addr: AccountAddress,
    //TODO security, this field should keep in memory?
    private_key: Ed25519PrivateKey,
    store: AccountStorage,
}

impl Account {
    pub fn create(
        public_key: Ed25519PublicKey,
        private_key: Ed25519PrivateKey,
        addr: Option<AccountAddress>,
        password: String,
        storage: AccountStorage,
    ) -> AccountResult<Self> {
        let address = addr.unwrap_or_else(|| account_address::from_public_key(&public_key));
        storage.update_key(address, public_key, &private_key, password.as_str())?;

        Ok(Self {
            addr: address,
            private_key,
            store: storage,
        })
    }

    pub fn load(
        addr: AccountAddress,
        password: &str,
        store: AccountStorage,
    ) -> AccountResult<Option<Self>> {
        let decrypted_key = store.decrypt_private_key(addr, password)?;
        let private_key = match decrypted_key {
            None => return Ok(None),
            Some(p) => p,
        };

        let saved_public_key = store.public_key(addr)?;
        let saved_public_key = saved_public_key.ok_or_else(|| {
            AccountError::StoreError(format_err!("public key not found for address {}", addr))
        })?;
        if saved_public_key.to_bytes() != private_key.public_key().to_bytes() {
            return Err(AccountError::StoreError(format_err!(
                "invalid state of public key and private key"
            )));
        }

        Ok(Some(Self {
            addr,
            private_key,
            store,
        }))
    }

    pub fn info(&self) -> AccountInfo {
        // TODO: fix is_default
        AccountInfo::new(self.addr, self.private_key.public_key(), false)
    }

    pub fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        raw_txn
            .sign(&self.private_key, self.private_key.public_key())
            .map(|t| t.into_inner())
    }

    pub fn destroy(self) -> Result<()> {
        self.store.destroy_account(self.addr)
    }

    pub fn address(&self) -> &AccountAddress {
        &self.addr
    }

    pub fn private_key(&self) -> &Ed25519PrivateKey {
        &self.private_key
    }

    pub fn public_key(&self) -> Ed25519PublicKey {
        //TODO should keep a public_key cache?
        self.private_key.public_key()
    }

    pub fn get_auth_key(&self) -> AuthenticationKey {
        AuthenticationKey::ed25519(&self.public_key())
    }

    ///Generate a random account for test.
    pub fn random() -> Result<Self> {
        let mut key_gen = KeyGen::from_os_rng();
        let (private_key, public_key) = key_gen.generate_keypair();
        let address = account_address::from_public_key(&public_key);
        let storage = AccountStorage::new(StorageInstance::new_cache_instance());
        Self::create(
            public_key,
            private_key,
            Some(address),
            "".to_string(),
            storage,
        )
        .map_err(|e| e.into())
    }
}
