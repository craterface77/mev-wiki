use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use revm::context::{BlockEnv, TxEnv};
use revm::context_interface::result::{ExecutionResult, Output};
use revm::handler::{ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainnetContext};
use revm::primitives::{hardfork::SpecId, TxKind};
use revm::state::Bytecode;

use crate::db::ForkDb;
use crate::error::{SimulateError, SimulateResult};
use crate::types::{BlockCtx, SimulateTx};

pub struct CallResult {
    pub output:   Bytes,
    pub gas_used: u64,
    pub logs:     Vec<revm::primitives::Log>,
}

pub struct SimulateExecutor<P: Provider + Clone + Send + Sync + 'static> {
    pub db: ForkDb<P>,
}

impl<P: Provider + Clone + Send + Sync + 'static> SimulateExecutor<P> {
    pub fn new(db: ForkDb<P>) -> Self {
        Self { db }
    }

    fn make_block_env(block_ctx: &BlockCtx) -> BlockEnv {
        BlockEnv {
            number:      U256::from(block_ctx.number),
            beneficiary: block_ctx.coinbase,
            timestamp:   U256::from(block_ctx.timestamp),
            basefee:     block_ctx.base_fee.saturating_to::<u64>(),
            gas_limit:   u64::MAX,
            ..Default::default()
        }
    }

    fn make_tx_env(tx: &SimulateTx) -> TxEnv {
        TxEnv {
            caller:          tx.caller,
            kind:            TxKind::Call(tx.to),
            data:            tx.calldata.clone(),
            value:           tx.value,
            gas_limit:       tx.gas_limit,
            gas_price:       tx.gas_price.saturating_to::<u128>(),
            gas_priority_fee: tx.priority_fee.map(|f| f.saturating_to::<u128>()),
            chain_id:        None,
            nonce:           0,
            ..Default::default()
        }
    }

    fn build_ctx(&mut self, block_ctx: &BlockCtx) -> MainnetContext<&mut ForkDb<P>> {
        let block = Self::make_block_env(block_ctx);
        let mut ctx = MainnetContext::new(&mut self.db, SpecId::CANCUN)
            .with_block(block);
        ctx.modify_cfg(|cfg| {
            cfg.disable_base_fee    = true;
            cfg.disable_nonce_check = true;
        });
        ctx
    }

    pub fn call_commit(&mut self, block_ctx: &BlockCtx, tx: &SimulateTx) -> SimulateResult<CallResult> {
        let tx_env = Self::make_tx_env(tx);
        let result = self
            .build_ctx(block_ctx)
            .build_mainnet()
            .transact_commit(tx_env)
            .map_err(|e| SimulateError::Database(format!("{e:?}")))?;
        Self::parse_result(result)
    }

    pub fn call_ref(&mut self, block_ctx: &BlockCtx, tx: &SimulateTx) -> SimulateResult<CallResult> {
        let tx_env = Self::make_tx_env(tx);
        let result = self
            .build_ctx(block_ctx)
            .build_mainnet()
            .transact(tx_env)
            .map_err(|e| SimulateError::Database(format!("{e:?}")))?
            .result;
        Self::parse_result(result)
    }

    #[inline]
    pub fn set_eth_balance(&mut self, address: Address, balance: U256) {
        self.db.set_eth_balance(address, balance);
    }

    #[inline]
    pub fn set_storage(&mut self, address: Address, slot: U256, value: U256) -> SimulateResult<()> {
        self.db.set_storage(address, slot, value)
    }

    pub fn set_token_balance(
        &mut self,
        account: Address,
        token: Address,
        amount: U256,
    ) -> SimulateResult<()> {
        use crate::profit_calculator::{balance_slot, known_balance_slot};
        let slot_idx = known_balance_slot(token).unwrap_or(0);
        let slot_key = balance_slot(account, slot_idx);
        self.db.set_storage(token, U256::from_be_bytes(*slot_key), amount)
    }

    #[inline]
    pub fn deploy(&mut self, address: Address, bytecode: Bytecode) {
        self.db.deploy_contract(address, bytecode);
    }

    #[inline]
    pub fn snapshot(&self) -> ForkDb<P> {
        self.db.clone()
    }

    #[inline]
    pub fn restore(&mut self, snapshot: ForkDb<P>) {
        self.db = snapshot;
    }

    fn parse_result(result: ExecutionResult) -> SimulateResult<CallResult> {
        match result {
            ExecutionResult::Success { output, gas, logs, .. } => {
                let bytes = match output {
                    Output::Call(b)      => b,
                    Output::Create(b, _) => b,
                };
                Ok(CallResult { output: bytes, gas_used: gas.spent(), logs })
            }
            ExecutionResult::Revert { output, gas, .. } => Err(SimulateError::Revert(format!(
                "gas_used={} data=0x{}",
                gas.spent(),
                hex::encode(&output)
            ))),
            ExecutionResult::Halt { reason, gas, .. } => {
                Err(SimulateError::Halt(format!("{reason:?} gas_used={}", gas.spent())))
            }
        }
    }
}

pub fn check_trap_token<P: Provider + Clone + Send + Sync + 'static>(
    executor: &mut SimulateExecutor<P>,
    block_ctx: &BlockCtx,
    token: Address,
    test_sender: Address,
    test_recipient: Address,
    balance_slot: U256,
    amount: U256,
) -> SimulateResult<bool> {
    let slot = keccak_erc20_balance_slot(test_sender, balance_slot);
    executor.set_storage(token, slot, amount)?;

    let _ = executor.call_ref(block_ctx, &SimulateTx {
        caller:       test_sender,
        to:           token,
        calldata:     Bytes::from(encode_transfer(test_recipient, amount)),
        value:        U256::ZERO,
        gas_limit:    200_000,
        gas_price:    U256::ZERO,
        priority_fee: None,
    })?;

    let result = executor.call_ref(block_ctx, &SimulateTx {
        caller:       Address::ZERO,
        to:           token,
        calldata:     Bytes::from(encode_balance_of(test_recipient)),
        value:        U256::ZERO,
        gas_limit:    100_000,
        gas_price:    U256::ZERO,
        priority_fee: None,
    })?;

    if result.output.len() < 32 {
        return Ok(true);
    }
    let received  = U256::from_be_slice(&result.output[..32]);
    let threshold = amount * U256::from(99u64) / U256::from(100u64);
    Ok(received < threshold)
}

fn keccak_erc20_balance_slot(address: Address, slot: U256) -> U256 {
    let mut data = [0u8; 64];
    data[12..32].copy_from_slice(address.as_slice());
    data[32..64].copy_from_slice(&slot.to_be_bytes::<32>());
    let hash = revm::primitives::keccak256(&data);
    U256::from_be_slice(&hash.0)
}

fn encode_transfer(to: Address, amount: U256) -> Vec<u8> {
    let mut v = Vec::with_capacity(68);
    v.extend_from_slice(&[0xa9, 0x05, 0x9c, 0xbb]);
    v.extend_from_slice(&[0u8; 12]);
    v.extend_from_slice(to.as_slice());
    v.extend_from_slice(&amount.to_be_bytes::<32>());
    v
}

fn encode_balance_of(account: Address) -> Vec<u8> {
    let mut v = Vec::with_capacity(36);
    v.extend_from_slice(&[0x70, 0xa0, 0x82, 0x31]);
    v.extend_from_slice(&[0u8; 12]);
    v.extend_from_slice(account.as_slice());
    v
}
