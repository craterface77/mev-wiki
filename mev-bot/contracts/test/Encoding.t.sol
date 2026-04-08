// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "./misc/Encoding.sol";

/// @notice Pure encoding unit tests — no fork, no Huff deploy needed.
contract EncodingTest is Test {

    function testFiveByteEncoding_1ETH() public pure {
        uint256 amount  = 1 ether;
        bytes5  encoded = FiveByteEncoding.encode(amount);
        uint256 decoded = FiveByteEncoding.decode(encoded);
        assertApproxEqRel(decoded, amount, 1e10, "5-byte 1 ETH roundtrip error > 1e10 wei");
    }

    function testFiveByteEncoding_Small() public pure {
        uint256 amount  = 1000e6; // 1000 USDC (6 decimals)
        bytes5  encoded = FiveByteEncoding.encode(amount);
        uint256 decoded = FiveByteEncoding.decode(encoded);
        // Max loss = 1 byte shift = 256 units
        assertApproxEqAbs(decoded, amount, 256, "5-byte USDC roundtrip error too large");
    }

    function testFiveByteEncoding_Large() public pure {
        uint256 amount  = 100_000 ether;
        bytes5  encoded = FiveByteEncoding.encode(amount);
        uint256 decoded = FiveByteEncoding.decode(encoded);
        // Relative error < 0.001%
        assertApproxEqRel(decoded, amount, 1e13, "5-byte large amount error too large");
    }

    function testFuzz_FiveByteEncoding(uint256 amount) public pure {
        amount = bound(amount, 1, type(uint128).max);
        bytes5  encoded = FiveByteEncoding.encode(amount);
        uint256 decoded = FiveByteEncoding.decode(encoded);
        // decoded >= amount - 256^4 (max 4-byte truncation)
        // For amounts > 1e9, relative error < 1/256
        if (amount > 1e9) {
            assertApproxEqRel(decoded, amount, 1e16, "5-byte fuzz error > 0.01%");
        } else {
            assertApproxEqAbs(decoded, amount, 256, "5-byte fuzz small amount error");
        }
    }

    function testWethEncoding_roundtrip(uint256 wethAmount) public pure {
        wethAmount = bound(wethAmount, 1e5, 1000 ether);
        uint256 txValue  = WethEncoding.encode(wethAmount);
        uint256 decoded  = WethEncoding.decode(txValue);
        // Loss = last 5 digits = at most 99999 wei
        assertApproxEqAbs(decoded, wethAmount, 1e5, "WETH encoding loss > 1e5 wei");
    }

    function testWethEncoding_1ETH() public pure {
        uint256 encoded = WethEncoding.encode(1 ether);
        assertEq(encoded, 1 ether / 1e5, "WETH encode 1 ETH wrong");
        assertEq(WethEncoding.decode(encoded), 1 ether, "WETH decode 1 ETH wrong");
    }

    function testJumpDestTable() public pure {
        // Verify JUMPDEST offsets match contract layout
        // Layout: 5 bytes per gate, starting at 0x05
        assertEq(SandoBotCalldata.JD_V2_BACKRUN0,  0x05);
        assertEq(SandoBotCalldata.JD_V2_FRONTRUN0, 0x0A);
        assertEq(SandoBotCalldata.JD_V2_BACKRUN1,  0x0F);
        assertEq(SandoBotCalldata.JD_V2_FRONTRUN1, 0x14);
        assertEq(SandoBotCalldata.JD_V3_BACKRUN0,  0x19);
        assertEq(SandoBotCalldata.JD_V3_FRONTRUN0, 0x1E);
        assertEq(SandoBotCalldata.JD_V3_BACKRUN1,  0x23);
        assertEq(SandoBotCalldata.JD_V3_FRONTRUN1, 0x28);
        assertEq(SandoBotCalldata.JD_RECOVER_WETH, 0x2D);
        assertEq(SandoBotCalldata.JD_RECOVER_ETH,  0x32);
        assertEq(SandoBotCalldata.JD_SEPPUKU,      0x37);
    }

    function testV2Frontrun0Calldata() public pure {
        address pair     = address(0x1234);
        uint256 wethIn   = 5 ether;
        uint256 tokenOut = 10_000e6; // 10k USDC

        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Frontrun0Full(pair, wethIn, tokenOut);

        // First byte = JUMPDEST
        assertEq(uint8(cd[0]), SandoBotCalldata.JD_V2_FRONTRUN0, "wrong jumpdest");
        // Bytes 1-20 = pair address (right-padded in calldata)
        // txVal = wethIn / 1e5
        assertEq(txVal, wethIn / 1e5, "wrong tx value");
        // Total length: 1 (jumpdest) + 20 (pair) + 5 (encoded) = 26
        assertEq(cd.length, 26, "wrong calldata length for v2 frontrun0");
    }

    function testV2Backrun0Calldata() public pure {
        address pair     = address(0x1234);
        address tokenIn  = address(0x5678);
        uint256 amountIn = 10_000e6;
        uint256 wethOut  = 5 ether;

        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Backrun0Full(pair, tokenIn, amountIn, wethOut);

        assertEq(uint8(cd[0]), SandoBotCalldata.JD_V2_BACKRUN0, "wrong jumpdest");
        assertEq(txVal, wethOut / 1e5, "wrong tx value");
        // 1 + 20 (pair) + 20 (tokenIn) + 5 (encoded) = 46
        assertEq(cd.length, 46, "wrong calldata length for v2 backrun0");
    }

    function testV3Frontrun1Calldata() public pure {
        address pool        = address(0xABCD);
        bytes32 poolKeyHash = keccak256("test");
        uint256 wethIn      = 1 ether;

        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v3Frontrun1Full(pool, poolKeyHash, wethIn);

        assertEq(uint8(cd[0]), SandoBotCalldata.JD_V3_FRONTRUN1, "wrong jumpdest");
        assertEq(txVal, wethIn / 1e5, "wrong tx value");
        // 1 + 20 (pool) + 32 (poolKeyHash) = 53
        assertEq(cd.length, 53, "wrong calldata length for v3 frontrun1");
    }
}
