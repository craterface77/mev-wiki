use alloy::primitives::{Address, Bytes, U256};

// IUniswapV2Pair.swap(uint256,uint256,address,bytes)
const SEL_V2_SWAP: [u8; 4] = [0x02, 0x2c, 0x0d, 0x9f];

// IUniswapV3Pool.swap(address,bool,int256,uint160,bytes)
const SEL_V3_SWAP: [u8; 4] = [0x12, 0x84, 0x09, 0x36];

// sqrt price limits — no limit sentinels
const MIN_SQRT_RATIO_PLUS_ONE: U256 = U256::from_limbs([4295128740, 0, 0, 0]);
const MAX_SQRT_RATIO_MINUS_ONE: U256 = U256::from_limbs([
    0xfffd8963efd1fc42,
    0xffffbf57fab5fffb,
    0xffffffffffffffff,
    0x0000000000000000,
]);

/// IUniswapV2Pair.swap(amount0Out, amount1Out, to, data)
pub fn encode_v2_swap(amount_out: U256, zero_for_one: bool, recipient: Address) -> Bytes {
    let (amount0_out, amount1_out) = if zero_for_one {
        (U256::ZERO, amount_out)
    } else {
        (amount_out, U256::ZERO)
    };

    let mut buf = Vec::with_capacity(4 + 4 * 32);
    buf.extend_from_slice(&SEL_V2_SWAP);
    buf.extend_from_slice(&pad32(amount0_out));
    buf.extend_from_slice(&pad32(amount1_out));
    buf.extend_from_slice(&pad_address(recipient));
    buf.extend_from_slice(&pad32(U256::from(128u64)));
    buf.extend_from_slice(&pad32(U256::ZERO));
    Bytes::from(buf)
}

/// IUniswapV3Pool.swap(recipient, zeroForOne, amountSpecified, sqrtPriceLimitX96, data)
pub fn encode_v3_swap(amount_in: U256, zero_for_one: bool, recipient: Address) -> Bytes {
    let sqrt_price_limit = if zero_for_one {
        MIN_SQRT_RATIO_PLUS_ONE
    } else {
        MAX_SQRT_RATIO_MINUS_ONE
    };

    let mut buf = Vec::with_capacity(4 + 5 * 32 + 64);
    buf.extend_from_slice(&SEL_V3_SWAP);
    buf.extend_from_slice(&pad_address(recipient));
    buf.extend_from_slice(&pad_bool(zero_for_one));
    buf.extend_from_slice(&pad32(amount_in));
    buf.extend_from_slice(&pad32(sqrt_price_limit));
    buf.extend_from_slice(&pad32(U256::from(160u64)));
    buf.extend_from_slice(&pad32(U256::ZERO));
    Bytes::from(buf)
}

#[inline]
fn pad32(val: U256) -> [u8; 32] { val.to_be_bytes() }

#[inline]
fn pad_address(addr: Address) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[12..32].copy_from_slice(addr.as_slice());
    buf
}

#[inline]
fn pad_bool(val: bool) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[31] = val as u8;
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn v2_swap_zero_for_one() {
        let amount_out = U256::from(1000u64);
        let recipient  = address!("DeadDeadDeadDeadDeadDeadDeadDeadDeadDead");
        let data       = encode_v2_swap(amount_out, true, recipient);

        assert_eq!(&data[0..4], &SEL_V2_SWAP);
        assert_eq!(&data[4..36], &[0u8; 32]);
        let mut expected = [0u8; 32];
        expected[31] = 0xe8;
        expected[30] = 0x03;
        assert_eq!(&data[36..68], &expected);
    }

    #[test]
    fn v2_swap_one_for_zero() {
        let amount_out = U256::from(500u64);
        let recipient  = address!("DeadDeadDeadDeadDeadDeadDeadDeadDeadDead");
        let data       = encode_v2_swap(amount_out, false, recipient);

        assert_eq!(&data[0..4], &SEL_V2_SWAP);
        let mut expected = [0u8; 32];
        expected[31] = 0xf4;
        expected[30] = 0x01;
        assert_eq!(&data[4..36], &expected);
        assert_eq!(&data[36..68], &[0u8; 32]);
    }

    #[test]
    fn v3_swap_encoding_length() {
        let data = encode_v3_swap(
            U256::from(1_000_000u64),
            true,
            address!("DeadDeadDeadDeadDeadDeadDeadDeadDeadDead"),
        );
        assert_eq!(data.len(), 4 + 6 * 32);
        assert_eq!(&data[0..4], &SEL_V3_SWAP);
    }
}
