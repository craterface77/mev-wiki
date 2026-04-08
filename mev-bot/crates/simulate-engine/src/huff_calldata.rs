use alloy::primitives::{Address, Bytes, B256, U256};

pub const JD_V2_BACKRUN0:  u8 = 0x05;
pub const JD_V2_FRONTRUN0: u8 = 0x0A;
pub const JD_V2_BACKRUN1:  u8 = 0x0F;
pub const JD_V2_FRONTRUN1: u8 = 0x14;
pub const JD_V3_BACKRUN0:  u8 = 0x19;
pub const JD_V3_FRONTRUN0: u8 = 0x1E;
pub const JD_V3_BACKRUN1:  u8 = 0x23;
pub const JD_V3_FRONTRUN1: u8 = 0x28;
pub const JD_RECOVER_WETH: u8 = 0x2D;
pub const JD_RECOVER_ETH:  u8 = 0x32;
pub const JD_SEPPUKU:      u8 = 0x37;

const WETH_VALUE_DIVISOR: u128 = 100_000;

#[inline]
pub fn encode_weth_value(weth_amount: U256) -> U256 {
    weth_amount / U256::from(WETH_VALUE_DIVISOR)
}

/// Packs amount into 5 bytes: [memOffset:1][fourByteValue:4]
/// Decoded by Huff via: mstore(memOffset, fourByteValue)
pub fn encode_huff_amount(amount: U256) -> [u8; 5] {
    let bytes = amount.to_be_bytes::<32>();
    let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(28);
    let mem_offset = first_nonzero.min(28) as u8;
    let mut encoded = [0u8; 4];
    let src = &bytes[mem_offset as usize..];
    let copy_len = 4.min(src.len());
    encoded[..copy_len].copy_from_slice(&src[..copy_len]);
    [mem_offset, encoded[0], encoded[1], encoded[2], encoded[3]]
}

pub fn v2_frontrun_weth0(pair: Address, token_out_amount: U256) -> Bytes {
    let mut buf = Vec::with_capacity(26);
    buf.push(JD_V2_FRONTRUN0);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(&encode_huff_amount(token_out_amount));
    Bytes::from(buf)
}

pub fn v2_frontrun_weth1(pair: Address, token_out_amount: U256) -> Bytes {
    let mut buf = Vec::with_capacity(26);
    buf.push(JD_V2_FRONTRUN1);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(&encode_huff_amount(token_out_amount));
    Bytes::from(buf)
}

pub fn v2_backrun_weth0(pair: Address, token_in: Address, amount_in: U256) -> Bytes {
    let mut buf = Vec::with_capacity(46);
    buf.push(JD_V2_BACKRUN0);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(token_in.as_slice());
    buf.extend_from_slice(&encode_huff_amount(amount_in));
    Bytes::from(buf)
}

pub fn v2_backrun_weth1(pair: Address, token_in: Address, amount_in: U256) -> Bytes {
    let mut buf = Vec::with_capacity(46);
    buf.push(JD_V2_BACKRUN1);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(token_in.as_slice());
    buf.extend_from_slice(&encode_huff_amount(amount_in));
    Bytes::from(buf)
}

pub fn v3_frontrun_weth0(pair: Address, pool_key_hash: B256) -> Bytes {
    let mut buf = Vec::with_capacity(53);
    buf.push(JD_V3_FRONTRUN0);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(pool_key_hash.as_slice());
    Bytes::from(buf)
}

pub fn v3_frontrun_weth1(pair: Address, pool_key_hash: B256) -> Bytes {
    let mut buf = Vec::with_capacity(53);
    buf.push(JD_V3_FRONTRUN1);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(pool_key_hash.as_slice());
    Bytes::from(buf)
}

pub fn v3_backrun_weth0(pair: Address, token_in: Address, pool_key_hash: B256, amount_in: U256) -> Bytes {
    let mut buf = Vec::with_capacity(78);
    buf.push(JD_V3_BACKRUN0);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(token_in.as_slice());
    buf.extend_from_slice(pool_key_hash.as_slice());
    buf.extend_from_slice(&encode_huff_amount(amount_in));
    Bytes::from(buf)
}

pub fn v3_backrun_weth1(pair: Address, token_in: Address, pool_key_hash: B256, amount_in: U256) -> Bytes {
    let mut buf = Vec::with_capacity(78);
    buf.push(JD_V3_BACKRUN1);
    buf.extend_from_slice(pair.as_slice());
    buf.extend_from_slice(token_in.as_slice());
    buf.extend_from_slice(pool_key_hash.as_slice());
    buf.extend_from_slice(&encode_huff_amount(amount_in));
    Bytes::from(buf)
}

use crate::types::{DexVersion, PoolState};

pub struct HuffBundle {
    pub front_calldata: Bytes,
    pub front_value:    U256,
    pub back_calldata:  Bytes,
    pub back_value:     U256,
}

pub fn build_huff_bundle(
    dex_version:   DexVersion,
    _pool_state:   &PoolState,
    pair:          Address,
    _token_in:     Address,
    token_out:     Address,
    zero_for_one:  bool,
    front_weth_in: U256,
    front_out:     U256,
    pool_key_hash: B256,
) -> Option<HuffBundle> {
    let front_value = encode_weth_value(front_weth_in);
    let back_value  = encode_weth_value(front_out);

    let (front_calldata, back_calldata) = match dex_version {
        DexVersion::UniswapV2 | DexVersion::SushiswapV2 | DexVersion::PancakeV2 => {
            if zero_for_one {
                (v2_frontrun_weth0(pair, front_out), v2_backrun_weth0(pair, token_out, front_out))
            } else {
                (v2_frontrun_weth1(pair, front_out), v2_backrun_weth1(pair, token_out, front_out))
            }
        }
        DexVersion::UniswapV3 | DexVersion::SushiswapV3 | DexVersion::PancakeV3 => {
            if zero_for_one {
                (v3_frontrun_weth0(pair, pool_key_hash), v3_backrun_weth0(pair, token_out, pool_key_hash, front_out))
            } else {
                (v3_frontrun_weth1(pair, pool_key_hash), v3_backrun_weth1(pair, token_out, pool_key_hash, front_out))
            }
        }
        DexVersion::UniswapV4 | DexVersion::CurveStable => return None,
    };

    Some(HuffBundle { front_calldata, front_value, back_calldata, back_value })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_encode_huff_amount_1eth() {
        let one_eth = U256::from(1_000_000_000_000_000_000u128);
        let encoded = encode_huff_amount(one_eth);
        let mem_offset = encoded[0] as usize;
        let val = u32::from_be_bytes([encoded[1], encoded[2], encoded[3], encoded[4]]);
        let restored = U256::from(val) << (8 * (32 - mem_offset - 4));
        assert_eq!(one_eth, restored);
    }

    #[test]
    fn test_encode_weth_value() {
        let weth = U256::from(1_500_000_000_000_000_000u128);
        let val  = encode_weth_value(weth);
        let restored = val * U256::from(100_000u64);
        let diff = weth.saturating_sub(restored);
        assert!(diff < U256::from(100_000u64));
    }

    #[test]
    fn test_v2_frontrun_calldata_length() {
        let pair = address!("0000000000000000000000000000000000000001");
        let cd = v2_frontrun_weth0(pair, U256::from(1000u64));
        assert_eq!(cd.len(), 26);
        assert_eq!(cd[0], JD_V2_FRONTRUN0);
    }

    #[test]
    fn test_v2_backrun_calldata_length() {
        let pair     = address!("0000000000000000000000000000000000000001");
        let token_in = address!("0000000000000000000000000000000000000002");
        let cd = v2_backrun_weth0(pair, token_in, U256::from(1000u64));
        assert_eq!(cd.len(), 46);
        assert_eq!(cd[0], JD_V2_BACKRUN0);
    }

    #[test]
    fn test_v3_frontrun_calldata_length() {
        let pair = address!("0000000000000000000000000000000000000001");
        let cd = v3_frontrun_weth0(pair, B256::ZERO);
        assert_eq!(cd.len(), 53);
        assert_eq!(cd[0], JD_V3_FRONTRUN0);
    }
}
