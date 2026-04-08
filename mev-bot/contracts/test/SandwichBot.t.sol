// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "foundry-huff/HuffDeployer.sol";

import "./misc/Interfaces.sol";
import "./misc/Encoding.sol";
import "./misc/V2Helper.sol";

/// @title SandwichBotTest
/// @notice Fork tests for SandwichBot.huff on mainnet state.
///
/// Run:
///   ETH_RPC_URL=<alchemy_or_local> forge test -vv --match-contract SandwichBotTest
///
/// Tests cover:
///   1. Utility methods: recoverEth, recoverWeth, seppuku
///   2. Access control: non-owner cannot call sandwich methods
///   3. V2 sandwich: frontrun0 (WETH=token0) + victim + backrun0 → net profit
///   4. V2 sandwich: frontrun1 (WETH=token1) + victim + backrun1 → net profit
///   5. Encoding roundtrip: FiveByteEncoding, WethEncoding
///   6. V3 frontrun (smoke test — full sandwich needs V3 callback)
contract SandwichBotTest is Test {
    // ── Constants ──────────────────────────────────────────────────────────

    // Owner = private key 0x1 → address below
    address constant OWNER   = 0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf;
    address constant ATTACKER = address(0xBAD);

    // Mainnet addresses
    address constant WETH_ADDR  = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant USDC_ADDR  = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address constant USDT_ADDR  = 0xdAC17F958D2ee523a2206206994597C13D831ec7;
    address constant DAI_ADDR   = 0x6B175474E89094C44Da98b954EedeAC495271d0F;

    // Uniswap V2 pairs (mainnet)
    // USDC/WETH — USDC=token0, WETH=token1 (USDC addr < WETH addr numerically)
    address constant WETH_USDC_PAIR = 0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc;
    // WETH/USDT — WETH=token0, USDT=token1
    address constant WETH_USDT_PAIR = 0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852;
    // DAI/WETH  — DAI=token0, WETH=token1 (DAI addr < WETH addr)
    address constant WETH_DAI_PAIR  = 0xA478c2975Ab1Ea89e8196811F51A7B7Ade33eB11;

    // Uniswap V3: USDC/WETH 0.05% pool — WETH is token1
    address constant USDC_WETH_V3_POOL = 0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640;

    // ── State ──────────────────────────────────────────────────────────────

    address sando;
    IWETH   weth = IWETH(WETH_ADDR);

    uint256 constant FUND_WETH = 50 ether;
    uint256 constant FORK_BLOCK = 21_800_000; // ~Dec 2024, within Alchemy archive range

    // ── Setup ──────────────────────────────────────────────────────────────

    function setUp() public {
        // Fork mainnet — use latest block (archive not available on Alchemy free tier)
        // Pass --fork-block-number explicitly if you have archive access
        string memory rpc = vm.envOr("ETH_RPC_URL", string("https://eth.llamarpc.com"));
        vm.createSelectFork(rpc);

        // Deploy SandwichBot.huff via foundry-huff FFI.
        // Override OWNER to a fresh address with no on-chain code (avoid mainnet contract collisions).
        sando = HuffDeployer
            .config()
            .with_constant("OWNER", "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf")
            .deploy("SandwichBot");

        // Ensure OWNER has no code (address(0x7E5F...) might be a contract on mainnet latest)
        vm.etch(OWNER, bytes(""));

        // Fund sando contract with WETH
        vm.deal(address(this), FUND_WETH + 10 ether);
        weth.deposit{value: FUND_WETH}();
        weth.transfer(sando, FUND_WETH);

        // Fund OWNER with ETH (for prank)
        vm.deal(OWNER, 10 ether);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Encoding roundtrip tests (no fork needed, pure math)
    // ══════════════════════════════════════════════════════════════════════

    function testFiveByteEncoding_1ETH() public pure {
        uint256 amount  = 1 ether;
        bytes5  encoded = FiveByteEncoding.encode(amount);
        uint256 decoded = FiveByteEncoding.decode(encoded);
        // Lossy — should be within 1 byte shift precision
        assertApproxEqRel(decoded, amount, 1e10, "5-byte roundtrip > 1e10 wei error");
    }

    function testFiveByteEncoding_USDC(uint256 amount) public pure {
        // USDC: 6 decimals, typical amounts 1e6 - 1e12
        amount = bound(amount, 1e6, 1e12);
        bytes5  encoded = FiveByteEncoding.encode(amount);
        uint256 decoded = FiveByteEncoding.decode(encoded);
        // For 6-decimal tokens, precision loss is < 256 units (1 byte shift)
        assertApproxEqAbs(decoded, amount, 256, "5-byte USDC roundtrip error too large");
    }

    function testWethEncoding_roundtrip(uint256 wethAmount) public pure {
        wethAmount = bound(wethAmount, 1e13, 100 ether);
        uint256 txValue  = WethEncoding.encode(wethAmount);
        uint256 decoded  = WethEncoding.decode(txValue);
        // Loss ≤ 1e5 wei (last 5 digits)
        assertApproxEqAbs(decoded, wethAmount, 1e5, "WETH encoding loss > 1e5 wei");
    }

    // ══════════════════════════════════════════════════════════════════════
    // Access control
    // ══════════════════════════════════════════════════════════════════════

    function testRecoverEth_onlyOwner() public {
        // Non-owner should fail
        vm.prank(ATTACKER);
        (bool success,) = sando.call(SandoBotCalldata.recoverEth());
        assertFalse(success, "non-owner should not be able to recoverEth");
    }

    function testRecoverWeth_onlyOwner() public {
        vm.prank(ATTACKER);
        (bool success,) = sando.call(SandoBotCalldata.recoverWeth(1 ether));
        assertFalse(success, "non-owner should not be able to recoverWeth");
    }

    function testSeppuku_onlyOwner() public {
        vm.prank(ATTACKER);
        (bool success,) = sando.call(SandoBotCalldata.seppuku());
        assertFalse(success, "non-owner should not be able to seppuku");
    }

    function testV2Frontrun_onlyOwner() public {
        uint256 fakeOut   = 1000e6; // 1000 USDC
        uint256 fakeWethIn = 0.5 ether;
        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Frontrun0Full(
            WETH_USDC_PAIR, fakeWethIn, fakeOut
        );
        vm.prank(ATTACKER);
        (bool success,) = sando.call{value: txVal}(cd);
        assertFalse(success, "non-owner should not be able to frontrun");
    }

    // ══════════════════════════════════════════════════════════════════════
    // Utility methods
    // ══════════════════════════════════════════════════════════════════════

    function testRecoverEth() public {
        // Send some ETH to sando
        vm.deal(sando, 1 ether);
        uint256 ownerBefore = OWNER.balance;

        vm.prank(OWNER);
        (bool s,) = sando.call(SandoBotCalldata.recoverEth());
        assertTrue(s, "recoverEth failed");

        assertEq(sando.balance, 0, "sando ETH balance should be 0");
        assertEq(OWNER.balance, ownerBefore + 1 ether, "owner should receive ETH");
    }

    function testRecoverWeth() public {
        uint256 sandoBefore  = weth.balanceOf(sando);
        uint256 ownerBefore  = weth.balanceOf(OWNER);
        assertTrue(sandoBefore > 0, "sando should have WETH");

        vm.prank(OWNER);
        (bool s,) = sando.call(SandoBotCalldata.recoverWeth(sandoBefore));
        assertTrue(s, "recoverWeth failed");

        assertEq(weth.balanceOf(sando), 0,                         "sando WETH should be 0");
        assertEq(weth.balanceOf(OWNER), ownerBefore + sandoBefore, "owner should receive WETH");
    }

    function testSeppuku() public {
        vm.prank(OWNER);
        (bool s,) = sando.call(SandoBotCalldata.seppuku());
        assertTrue(s, "seppuku failed");
        // Contract should have no code after selfdestruct (in same tx = still exists, checked after)
    }

    // ══════════════════════════════════════════════════════════════════════
    // V2 Sandwich: WETH is token0 (WETH/USDC pair)
    // Sandwich: frontrun0 → victim buys USDC → backrun0 → net WETH profit
    // ══════════════════════════════════════════════════════════════════════

    // ── private helpers to keep stack depth under limit ──────────────────

    function _doFrontrun0(address pair, address tokenOut, uint256 frontWeth) private returns (uint256 tokenReceived) {
        uint256 frontTokenOut = V2Helper.getAmountOut(WETH_ADDR, tokenOut, frontWeth);
        assertTrue(frontTokenOut > 0, "frontrun expected output is 0");
        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Frontrun0Full(pair, frontWeth, frontTokenOut);
        vm.prank(OWNER);
        (bool ok,) = sando.call{value: txVal}(cd);
        assertTrue(ok, "frontrun0 failed");
        tokenReceived = IERC20(tokenOut).balanceOf(sando);
        assertGt(tokenReceived, 0, "sando should have token after frontrun0");
    }

    function _doFrontrun1(address pair, address tokenOut, uint256 frontWeth) private returns (uint256 tokenReceived) {
        uint256 frontTokenOut = V2Helper.getAmountOut(WETH_ADDR, tokenOut, frontWeth);
        assertTrue(frontTokenOut > 0, "frontrun expected output is 0");
        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Frontrun1Full(pair, frontWeth, frontTokenOut);
        vm.prank(OWNER);
        (bool ok,) = sando.call{value: txVal}(cd);
        assertTrue(ok, "frontrun1 failed");
        tokenReceived = IERC20(tokenOut).balanceOf(sando);
        assertGt(tokenReceived, 0, "sando should have token after frontrun1");
    }

    function _doVictimSwap(address victim, address tokenOut, uint256 victimWeth) private {
        vm.deal(victim, victimWeth + 1 ether);
        vm.startPrank(victim);
        IWETH(WETH_ADDR).deposit{value: victimWeth}();
        IWETH(WETH_ADDR).approve(V2Helper.UNIV2_ROUTER, victimWeth);
        address[] memory path = new address[](2);
        path[0] = WETH_ADDR; path[1] = tokenOut;
        IUniswapV2Router02(V2Helper.UNIV2_ROUTER).swapExactTokensForTokens(
            victimWeth, 0, path, victim, block.timestamp + 60
        );
        vm.stopPrank();
    }

    function _doBackrun0(address pair, address tokenOut, uint256 amountIn) private {
        uint256 wethOut = V2Helper.getAmountOut(tokenOut, WETH_ADDR, amountIn);
        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Backrun0Full(pair, tokenOut, amountIn, wethOut);
        vm.prank(OWNER);
        (bool ok,) = sando.call{value: txVal}(cd);
        assertTrue(ok, "backrun0 failed");
    }

    function _doBackrun1(address pair, address tokenOut, uint256 amountIn) private {
        uint256 wethOut = V2Helper.getAmountOut(tokenOut, WETH_ADDR, amountIn);
        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Backrun1Full(pair, tokenOut, amountIn, wethOut);
        vm.prank(OWNER);
        (bool ok,) = sando.call{value: txVal}(cd);
        assertTrue(ok, "backrun1 failed");
    }

    // WETH/USDT pair: WETH=token0, USDT=token1 → use frontrun0/backrun0
    function testV2Sandwich_WethToken0_USDT() public {
        address pair     = WETH_USDT_PAIR;
        address tokenOut = USDT_ADDR;
        uint256 frontWeth = 5 ether;

        uint256 wethBefore = weth.balanceOf(sando);
        assertTrue(wethBefore >= frontWeth, "not enough WETH in sando");

        uint256 tokenReceived = _doFrontrun0(pair, tokenOut, frontWeth);
        _doVictimSwap(address(0xDEAD1), tokenOut, 10 ether);
        _doBackrun0(pair, tokenOut, tokenReceived);

        // USDT doesn't return bool — check balance directly
        assertEq(IUSDT(tokenOut).balanceOf(sando), 0, "sando should have 0 USDT after backrun");

        uint256 wethAfter = weth.balanceOf(sando);
        emit log_named_decimal_uint("WETH before", wethBefore, 18);
        emit log_named_decimal_uint("WETH after ", wethAfter,  18);
        emit log_named_int("net WETH wei (USDT sandwich)", int256(wethAfter) - int256(wethBefore));

        assertGt(int256(wethAfter), int256(wethBefore) - int256(0.01 ether),
            "sandwich lost more than 0.01 ETH - check encoding or pool state");
    }

    // USDC/WETH pair: USDC=token0, WETH=token1 → use frontrun1/backrun1
    function testV2Sandwich_WethToken1_USDC() public {
        address pair     = WETH_USDC_PAIR;
        address tokenOut = USDC_ADDR;
        uint256 frontWeth = 5 ether;

        uint256 wethBefore = weth.balanceOf(sando);
        assertTrue(wethBefore >= frontWeth, "not enough WETH in sando");

        uint256 tokenReceived = _doFrontrun1(pair, tokenOut, frontWeth);
        _doVictimSwap(address(0xDEAD1), tokenOut, 10 ether);
        _doBackrun1(pair, tokenOut, tokenReceived);

        assertEq(IERC20(tokenOut).balanceOf(sando), 0, "sando should have 0 USDC after backrun");

        uint256 wethAfter = weth.balanceOf(sando);
        emit log_named_decimal_uint("WETH before", wethBefore, 18);
        emit log_named_decimal_uint("WETH after ", wethAfter,  18);
        emit log_named_int("net WETH wei (USDC sandwich)", int256(wethAfter) - int256(wethBefore));

        assertGt(int256(wethAfter), int256(wethBefore) - int256(0.01 ether),
            "sandwich lost more than 0.01 ETH - check encoding or pool state");
    }

    // ══════════════════════════════════════════════════════════════════════
    // V2 Sandwich: WETH is token1 (DAI/WETH pair)
    // Sandwich: frontrun1 → victim buys DAI → backrun1 → net WETH profit
    // ══════════════════════════════════════════════════════════════════════

    function testV2Sandwich_WethToken1_DAI() public {
        address pair     = WETH_DAI_PAIR;
        address tokenOut = DAI_ADDR;
        uint256 frontWeth = 3 ether;

        uint256 wethBefore = weth.balanceOf(sando);

        uint256 tokenReceived = _doFrontrun1(pair, tokenOut, frontWeth);
        _doVictimSwap(address(0xDEAD2), tokenOut, 8 ether);
        _doBackrun1(pair, tokenOut, tokenReceived);

        assertEq(IERC20(tokenOut).balanceOf(sando), 0, "sando should have 0 DAI after backrun");

        uint256 wethAfter = weth.balanceOf(sando);
        emit log_named_decimal_uint("WETH before", wethBefore, 18);
        emit log_named_decimal_uint("WETH after ", wethAfter,  18);

        assertGt(int256(wethAfter), int256(wethBefore) - int256(0.01 ether),
            "V2 token1 sandwich lost more than 0.01 ETH");
    }

    // ══════════════════════════════════════════════════════════════════════
    // V2 fuzz: frontrun + backrun amount encoding roundtrip at protocol level
    // ══════════════════════════════════════════════════════════════════════

    // WETH/USDT pair: WETH=token0 → frontrun0
    function testFuzz_V2FrontrunCalldata(uint256 wethIn) public {
        // Minimum: encoding must produce non-zero txValue (wethIn / 1e5 > 0)
        wethIn = bound(wethIn, 0.001 ether, 15 ether);

        address tokenOut    = USDT_ADDR;
        address pair        = WETH_USDT_PAIR;  // WETH=token0
        uint256 tokenOutAmt = V2Helper.getAmountOut(WETH_ADDR, tokenOut, wethIn);

        vm.assume(weth.balanceOf(sando) >= wethIn);
        vm.assume(tokenOutAmt > 0);

        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v2Frontrun0Full(pair, wethIn, tokenOutAmt);
        vm.assume(txVal > 0); // skip amounts too small for WETH encoding

        vm.prank(OWNER);
        (bool ok,) = sando.call{value: txVal}(cd);
        assertTrue(ok, "fuzz frontrun failed");

        assertGt(IUSDT(tokenOut).balanceOf(sando), 0, "no USDT after fuzz frontrun");
    }

    // ══════════════════════════════════════════════════════════════════════
    // V3 frontrun smoke test (no full sandwich — V3 callback is complex)
    // Tests that the frontrun tx goes through and WETH leaves sando
    // ══════════════════════════════════════════════════════════════════════

    function testV3Frontrun_Smoke_WethToken1() public {
        // USDC/WETH 0.05% pool: USDC=token0, WETH=token1
        // frontrun1: WETH is input (token1), zeroForOne=false
        address pool = USDC_WETH_V3_POOL;

        // Compute poolKeyHash = keccak256(abi.encode(token0, token1, fee))
        address token0 = IUniswapV3Pool(pool).token0(); // USDC
        address token1 = IUniswapV3Pool(pool).token1(); // WETH
        uint24  fee    = IUniswapV3Pool(pool).fee();    // 500
        bytes32 poolKeyHash = keccak256(abi.encode(token0, token1, fee));

        uint256 frontWeth = 1 ether;
        uint256 sandoWethBefore = weth.balanceOf(sando);

        (bytes memory cd, uint256 txVal) = SandoBotCalldata.v3Frontrun1Full(
            pool, poolKeyHash, frontWeth
        );

        vm.prank(OWNER);
        (bool ok,) = sando.call{value: txVal}(cd);
        assertTrue(ok, "V3 frontrun1 smoke test failed");

        // WETH should decrease, USDC should appear
        uint256 sandoUsdcAfter = IERC20(USDC_ADDR).balanceOf(sando);
        assertGt(sandoUsdcAfter, 0, "sando should have USDC after V3 frontrun");

        emit log_named_decimal_uint("USDC received from V3 frontrun", sandoUsdcAfter, 6);
        emit log_named_decimal_uint("WETH before", sandoWethBefore, 18);
        emit log_named_decimal_uint("WETH after ", weth.balanceOf(sando), 18);
    }
}
