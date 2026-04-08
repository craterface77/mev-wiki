use std::sync::Arc;

use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::BlockId;
use revm::database::CacheDB;
use revm::database_interface::{DBErrorMarker, Database, DatabaseCommit, DatabaseRef};
use revm::primitives::{AddressMap, KECCAK_EMPTY};
use revm::state::{Account, AccountInfo, Bytecode};

use crate::error::{SimulateError, SimulateResult};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct DbError(pub String);

impl DBErrorMarker for DbError {}

#[derive(Clone)]
pub struct RpcBackend<P: Provider + Clone + Send + Sync + 'static> {
    provider:     Arc<P>,
    block_number: u64,
    rt:           tokio::runtime::Handle,
}

impl<P: Provider + Clone + Send + Sync + 'static> RpcBackend<P> {
    pub fn new(provider: Arc<P>, block_number: u64) -> Self {
        Self {
            provider,
            block_number,
            rt: tokio::runtime::Handle::current(),
        }
    }

    fn block_id(&self) -> BlockId {
        BlockId::number(self.block_number)
    }

    fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        self.rt.block_on(f)
    }
}

impl<P: Provider + Clone + Send + Sync + 'static> DatabaseRef for RpcBackend<P> {
    type Error = DbError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let block = self.block_id();
        let p = &self.provider;

        let balance = self.block_on(async {
            p.get_balance(address).block_id(block).await
        }).map_err(|e| DbError(format!("balance: {e:?}")))?;

        let nonce = self.block_on(async {
            p.get_transaction_count(address).block_id(block).await
        }).map_err(|e| DbError(format!("nonce: {e:?}")))?;

        let code_bytes = self.block_on(async {
            p.get_code_at(address).block_id(block).await
        }).map_err(|e| DbError(format!("code: {e:?}")))?;

        let (code, code_hash) = if code_bytes.is_empty() {
            (None, KECCAK_EMPTY)
        } else {
            let bc = Bytecode::new_raw(code_bytes);
            let h  = bc.hash_slow();
            (Some(bc), h)
        };

        Ok(Some(AccountInfo {
            balance,
            nonce,
            code_hash,
            code,
            account_id: None,
        }))
    }

    fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::default())
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        let block = self.block_id();
        self.block_on(async {
            self.provider.get_storage_at(address, index).block_id(block).await
        })
        .map_err(|e| DbError(format!("storage: {e:?}")))
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        use alloy::eips::BlockNumberOrTag;
        let maybe_block = self.block_on(async {
            self.provider
                .get_block_by_number(BlockNumberOrTag::Number(number))
                .await
        })
        .map_err(|e| DbError(format!("block_hash: {e:?}")))?;

        Ok(maybe_block.map(|b| b.header.hash).unwrap_or_default())
    }
}

pub type InnerDb<P> = CacheDB<RpcBackend<P>>;

pub struct ForkDb<P: Provider + Clone + Send + Sync + 'static> {
    pub inner: InnerDb<P>,
}

impl<P: Provider + Clone + Send + Sync + 'static> ForkDb<P> {
    pub fn new(provider: Arc<P>, block_number: u64) -> Self {
        Self { inner: CacheDB::new(RpcBackend::new(provider, block_number)) }
    }

    pub fn set_eth_balance(&mut self, address: Address, balance: U256) {
        let info = AccountInfo {
            balance,
            nonce: 0,
            code_hash: KECCAK_EMPTY,
            code: None,
            account_id: None,
        };
        self.inner.insert_account_info(address, info);
    }

    pub fn set_storage(&mut self, address: Address, slot: U256, value: U256) -> SimulateResult<()> {
        self.inner
            .insert_account_storage(address, slot, value)
            .map_err(|e| SimulateError::Database(format!("{e:?}")))
    }

    pub fn read_storage(&mut self, address: Address, slot: U256) -> SimulateResult<U256> {
        use revm::database::Database;
        self.inner
            .storage(address, slot)
            .map_err(|e| SimulateError::Database(format!("{e:?}")))
    }

    pub fn deploy_contract(&mut self, address: Address, bytecode: Bytecode) {
        let info = AccountInfo {
            balance:    U256::ZERO,
            nonce:      1,
            code_hash:  bytecode.hash_slow(),
            code:       Some(bytecode),
            account_id: None,
        };
        self.inner.insert_account_info(address, info);
    }
}

impl<P: Provider + Clone + Send + Sync + 'static> Clone for ForkDb<P> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<P: Provider + Clone + Send + Sync + 'static> Database for ForkDb<P> {
    type Error = <InnerDb<P> as Database>::Error;
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.inner.basic(address)
    }
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.inner.code_by_hash(code_hash)
    }
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.inner.storage(address, index)
    }
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.inner.block_hash(number)
    }
}

impl<P: Provider + Clone + Send + Sync + 'static> DatabaseRef for ForkDb<P> {
    type Error = <InnerDb<P> as DatabaseRef>::Error;
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.inner.basic_ref(address)
    }
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.inner.code_by_hash_ref(code_hash)
    }
    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.inner.storage_ref(address, index)
    }
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.inner.block_hash_ref(number)
    }
}

impl<P: Provider + Clone + Send + Sync + 'static> DatabaseCommit for ForkDb<P> {
    fn commit(&mut self, changes: AddressMap<Account>) {
        self.inner.commit(changes)
    }
}
