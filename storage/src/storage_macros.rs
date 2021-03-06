// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// use crate::batch::WriteBatch;
// use crate::storage::InnerStorage;
// use crate::storage::StorageInstance;
// use crate::ColumnFamilyName;
// use std::sync::Arc;
#[macro_export]
macro_rules! define_storage {
    ($storage_type: ident, $key_type: ty, $value_type: ty, $prefix_name: expr) => {
        #[derive(Clone)]
        pub struct $storage_type {
            store: $crate::storage::CodecStorage<$key_type, $value_type>,
        }

        impl $storage_type {
            const PREFIX_NAME: $crate::storage::ColumnFamilyName = $prefix_name;
            pub fn new(instance: $crate::storage::StorageInstance) -> Self {
                let inner_storage = $crate::storage::InnerStorage::new(instance, Self::PREFIX_NAME);
                Self {
                    store: CodecStorage::new(std::sync::Arc::new(inner_storage)),
                }
            }
            pub fn put(&self, key: $key_type, value: $value_type) -> anyhow::Result<()> {
                self.store.put(key, value)
            }
            pub fn get(&self, key: $key_type) -> anyhow::Result<Option<$value_type>> {
                self.store.get(key)
            }
            pub fn remove(&self, key: $key_type) -> anyhow::Result<()> {
                self.store.remove(key)
            }
            pub fn write_batch(&self, batch: WriteBatch) -> anyhow::Result<()> {
                self.store.write_batch(batch)
            }
            pub fn get_len(&self) -> anyhow::Result<u64> {
                self.store.get_len()
            }
            pub fn keys(&self) -> anyhow::Result<Vec<Vec<u8>>> {
                self.store.keys()
            }
        }
    };
}
