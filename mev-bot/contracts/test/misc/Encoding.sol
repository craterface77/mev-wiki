// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @notice Реплика encoding логики из huff_calldata.rs и SandoCommon.sol.
///         Используется в тестах для построения calldata к SandwichBot.huff.

library WethEncoding {
    uint256 constant DIVISOR = 1e5;

    /// Кодировать WETH amount в tx.value
    function encode(uint256 amount) internal pure returns (uint256) {
        return amount / DIVISOR;
    }

    /// Декодировать tx.value обратно в WETH amount (lossy — теряются последние 5 цифр)
    function decode(uint256 value) internal pure returns (uint256) {
        return value * DIVISOR;
    }
}

library FiveByteEncoding {
    /// Кодирует amount в 5 байт: [memOffset:1][value:4]
    /// Идентично encode_huff_amount() в Rust
    function encode(uint256 amount) internal pure returns (bytes5) {
        uint8 byteShift = 0;
        uint32 fourByteValue = 0;

        while (byteShift < 32) {
            uint256 shifted = amount >> (8 * byteShift);
            if (shifted <= type(uint32).max) {
                fourByteValue = uint32(shifted);
                break;
            }
            byteShift++;
        }

        // memOffset for paramIndex=0 (amount0Out slot, starts at byte 4):
        //   memOffset = 4 + 32 - 4 - byteShift = 32 - byteShift
        // But huff mstore uses absolute memory offset and places 4-byte value at its END:
        //   mstore(memOffset, value) writes 32 bytes — value ends at memOffset+31
        //   we want value to end at: 4 + 32*(paramIndex+1) - 1
        //   so memOffset = 4 + 32*(paramIndex+1) - 1 - 31 - byteShift + byteShift
        //   simplifies to: 4 + 32*paramIndex + 28 - byteShift
        // paramIndex=0: 4 + 0 + 28 - byteShift = 32 - byteShift
        // paramIndex=1: 4 + 32 + 28 - byteShift = 64 - byteShift
        // Default encode() uses paramIndex=0 (amount0Out). Use encodeForParam() for paramIndex=1.
        uint8 memOffset = uint8(32 - byteShift);  // paramIndex=0 default

        return bytes5(abi.encodePacked(memOffset, fourByteValue));
    }

    /// Encode for a specific swap param slot (0=amount0Out, 1=amount1Out)
    function encodeForParam(uint256 amount, uint8 paramIndex) internal pure returns (bytes5) {
        uint8 byteShift = 0;
        uint32 fourByteValue = 0;
        while (byteShift < 32) {
            uint256 shifted = amount >> (8 * byteShift);
            if (shifted <= type(uint32).max) {
                fourByteValue = uint32(shifted);
                break;
            }
            byteShift++;
        }
        // memOffset = 4 + 32*paramIndex + 28 - byteShift
        uint8 memOffset = uint8(4 + 32 * uint256(paramIndex) + 28 - byteShift);
        return bytes5(abi.encodePacked(memOffset, fourByteValue));
    }

    /// Декодирует 5 байт обратно (lossy)
    /// memOffset = 4 + 32*paramIndex + 28 - byteShift
    /// For paramIndex=0: memOffset = 32 - byteShift → byteShift = 32 - memOffset
    function decode(bytes5 encoded) internal pure returns (uint256) {
        uint8  memOffset   = uint8(encoded[0]);
        uint32 fourByteVal = uint32(bytes4(encoded << 8));
        // byteShift = 32 - memOffset for paramIndex=0,
        // but for other indices memOffset > 32, so cap byteShift at 0
        uint8 byteShift = memOffset <= 32 ? uint8(32 - memOffset) : 0;
        return uint256(fourByteVal) << (8 * byteShift);
    }
}

/// @notice Строит calldata для SandwichBot.huff методов
library SandoBotCalldata {
    // JUMPDEST table — совпадает с SandwichBot.huff
    uint8 constant JD_V2_BACKRUN0  = 0x05;
    uint8 constant JD_V2_FRONTRUN0 = 0x0A;
    uint8 constant JD_V2_BACKRUN1  = 0x0F;
    uint8 constant JD_V2_FRONTRUN1 = 0x14;
    uint8 constant JD_V3_BACKRUN0  = 0x19;
    uint8 constant JD_V3_FRONTRUN0 = 0x1E;
    uint8 constant JD_V3_BACKRUN1  = 0x23;
    uint8 constant JD_V3_FRONTRUN1 = 0x28;
    uint8 constant JD_RECOVER_WETH = 0x2D;
    uint8 constant JD_RECOVER_ETH  = 0x32;
    uint8 constant JD_SEPPUKU      = 0x37;

    address constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

    // ── V2 Frontrun ───────────────────────────────────────────────

    /// WETH is token0 (input): [0x0A][pair:20][encodedTokenOut:5]
    function v2Frontrun0(address pair, uint256 tokenOutAmount)
        internal pure returns (bytes memory, uint256)
    {
        return (
            abi.encodePacked(JD_V2_FRONTRUN0, pair, FiveByteEncoding.encode(tokenOutAmount)),
            WethEncoding.encode(/* weth_in set by caller */ 0) // placeholder
        );
    }

    /// @param wethIn WETH amount bot sends; tokenOutAmount = expected output
    /// v2_frontrun0: WETH=token0 → swap(0, tokenOut, sando) → tokenOut in amount1Out slot (paramIndex=1)
    function v2Frontrun0Full(address pair, uint256 wethIn, uint256 tokenOutAmount)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V2_FRONTRUN0, pair, FiveByteEncoding.encodeForParam(tokenOutAmount, 1));
        txValue   = WethEncoding.encode(wethIn);
    }

    /// WETH is token1 (input): [0x14][pair:20][encodedTokenOut:5]
    /// v2_frontrun1: WETH=token1 → swap(tokenOut, 0, sando) → tokenOut in amount0Out slot (paramIndex=0)
    function v2Frontrun1Full(address pair, uint256 wethIn, uint256 tokenOutAmount)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V2_FRONTRUN1, pair, FiveByteEncoding.encodeForParam(tokenOutAmount, 0));
        txValue   = WethEncoding.encode(wethIn);
    }

    // ── V2 Backrun ────────────────────────────────────────────────

    /// WETH is token0 (output): [0x05][pair:20][tokenIn:20][encodedAmountIn:5]
    /// amountIn used in transfer(pair, amount) — amount is 2nd param (paramIndex=1)
    function v2Backrun0Full(address pair, address tokenIn, uint256 amountIn, uint256 wethOut)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V2_BACKRUN0, pair, tokenIn, FiveByteEncoding.encodeForParam(amountIn, 1));
        txValue   = WethEncoding.encode(wethOut);
    }

    /// WETH is token1 (output): [0x0F][pair:20][tokenIn:20][encodedAmountIn:5]
    function v2Backrun1Full(address pair, address tokenIn, uint256 amountIn, uint256 wethOut)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V2_BACKRUN1, pair, tokenIn, FiveByteEncoding.encodeForParam(amountIn, 1));
        txValue   = WethEncoding.encode(wethOut);
    }

    // ── V3 Frontrun ───────────────────────────────────────────────

    /// WETH is token0: [0x1E][pair:20][poolKeyHash:32]
    function v3Frontrun0Full(address pair, bytes32 poolKeyHash, uint256 wethIn)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V3_FRONTRUN0, pair, poolKeyHash);
        txValue   = WethEncoding.encode(wethIn);
    }

    /// WETH is token1: [0x28][pair:20][poolKeyHash:32]
    function v3Frontrun1Full(address pair, bytes32 poolKeyHash, uint256 wethIn)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V3_FRONTRUN1, pair, poolKeyHash);
        txValue   = WethEncoding.encode(wethIn);
    }

    // ── V3 Backrun ────────────────────────────────────────────────

    /// WETH is token0 (output): [0x19][pair:20][tokenIn:20][poolKeyHash:32][encodedAmountIn:5]
    function v3Backrun0Full(address pair, address tokenIn, bytes32 poolKeyHash, uint256 amountIn, uint256 wethOut)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V3_BACKRUN0, pair, tokenIn, poolKeyHash, FiveByteEncoding.encode(amountIn));
        txValue   = WethEncoding.encode(wethOut);
    }

    /// WETH is token1 (output): [0x23][pair:20][tokenIn:20][poolKeyHash:32][encodedAmountIn:5]
    function v3Backrun1Full(address pair, address tokenIn, bytes32 poolKeyHash, uint256 amountIn, uint256 wethOut)
        internal pure returns (bytes memory calldata_, uint256 txValue)
    {
        calldata_ = abi.encodePacked(JD_V3_BACKRUN1, pair, tokenIn, poolKeyHash, FiveByteEncoding.encode(amountIn));
        txValue   = WethEncoding.encode(wethOut);
    }

    // ── Utilities ─────────────────────────────────────────────────

    function recoverWeth(uint256 amount) internal pure returns (bytes memory) {
        return abi.encodePacked(JD_RECOVER_WETH, amount);
    }

    function recoverEth() internal pure returns (bytes memory) {
        return abi.encodePacked(JD_RECOVER_ETH);
    }

    function seppuku() internal pure returns (bytes memory) {
        return abi.encodePacked(JD_SEPPUKU);
    }
}
