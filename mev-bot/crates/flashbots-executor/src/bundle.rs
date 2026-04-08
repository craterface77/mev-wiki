use alloy::primitives::{Address, Bytes, U256};

#[derive(Debug, Clone)]
pub struct BundleTx {
    pub raw: Bytes,
}

#[derive(Debug, Clone)]
pub struct FlashbotsBundle {
    pub txs:          Vec<BundleTx>,
    pub block_number: u64,
}

impl FlashbotsBundle {
    pub fn new(block_number: u64) -> Self {
        Self { txs: Vec::new(), block_number }
    }

    pub fn push(&mut self, raw: Bytes) {
        self.txs.push(BundleTx { raw });
    }

    pub fn to_json_body(&self) -> serde_json::Value {
        let txs: Vec<String> = self.txs
            .iter()
            .map(|t| format!("0x{}", hex::encode(&t.raw)))
            .collect();

        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_sendBundle",
            "params": [{
                "txs": txs,
                "blockNumber": format!("0x{:x}", self.block_number),
                "revertingTxHashes": []
            }]
        })
    }
}

use alloy::consensus::{SignableTransaction, TxEip1559};
use alloy::network::TxSignerSync;
use alloy::signers::local::PrivateKeySigner;

pub fn sign_eip1559(
    signer:           &PrivateKeySigner,
    nonce:            u64,
    to:               Address,
    value:            U256,
    calldata:         Bytes,
    chain_id:         u64,
    max_fee_per_gas:  u128,
    max_priority_fee: u128,
    gas_limit:        u64,
) -> anyhow::Result<Bytes> {
    let mut tx = TxEip1559 {
        chain_id,
        nonce,
        gas_limit,
        max_fee_per_gas,
        max_priority_fee_per_gas: max_priority_fee,
        to: alloy::primitives::TxKind::Call(to),
        value,
        input: calldata,
        access_list: Default::default(),
    };

    let sig     = signer.sign_transaction_sync(&mut tx)?;
    let signed  = tx.into_signed(sig);
    let envelope = alloy::consensus::TxEnvelope::Eip1559(signed);

    let mut encoded = Vec::new();
    alloy::rlp::Encodable::encode(&envelope, &mut encoded);
    Ok(Bytes::from(encoded))
}

use simulate_engine::huff_calldata::HuffBundle;
use simulate_engine::types::SandwichCandidate;

pub struct SandwichBundleParams {
    pub bot_contract:       Address,
    pub searcher_signer:    PrivateKeySigner,
    pub chain_id:           u64,
    pub front_nonce:        u64,
    pub back_nonce:         u64,
    pub front_gas_limit:    u64,
    pub back_gas_limit:     u64,
    pub front_priority_fee: u128,
    pub back_priority_fee:  u128,
}

/// Bundle order: [frontrun, victim, backrun]
pub fn build_sandwich_bundle(
    params:       &SandwichBundleParams,
    huff:         &HuffBundle,
    victim_raw:   Bytes,
    candidate:    &SandwichCandidate,
    _gross_profit: u128,
) -> anyhow::Result<FlashbotsBundle> {
    let max_fee = candidate.block_ctx.max_fee.saturating_to::<u128>();

    let front_raw = sign_eip1559(
        &params.searcher_signer,
        params.front_nonce,
        params.bot_contract,
        huff.front_value,
        huff.front_calldata.clone(),
        params.chain_id,
        max_fee,
        params.front_priority_fee,
        params.front_gas_limit,
    )?;

    let back_raw = sign_eip1559(
        &params.searcher_signer,
        params.back_nonce,
        params.bot_contract,
        huff.back_value,
        huff.back_calldata.clone(),
        params.chain_id,
        max_fee,
        params.back_priority_fee,
        params.back_gas_limit,
    )?;

    let target_block = candidate.block_ctx.number + 1;

    let mut bundle = FlashbotsBundle::new(target_block);
    bundle.push(front_raw);
    bundle.push(victim_raw);
    bundle.push(back_raw);

    Ok(bundle)
}
