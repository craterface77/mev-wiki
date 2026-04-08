// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./Interfaces.sol";

/// @notice Вспомогательные функции для V2 тестов
library V2Helper {
    address constant WETH        = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    address constant UNIV2_FACTORY = 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f;
    address constant UNIV2_ROUTER  = 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D;

    /// Возвращает amountOut по формуле V2 AMM (0.3% fee)
    function getAmountOut(
        address tokenIn,
        address tokenOut,
        uint256 amountIn
    ) internal view returns (uint256 amountOut) {
        address pair   = IUniswapV2Factory(UNIV2_FACTORY).getPair(tokenIn, tokenOut);
        (uint112 r0, uint112 r1,) = IUniswapV2Pair(pair).getReserves();

        address t0 = IUniswapV2Pair(pair).token0();
        (uint256 rIn, uint256 rOut) = tokenIn == t0
            ? (uint256(r0), uint256(r1))
            : (uint256(r1), uint256(r0));

        uint256 amountInWithFee = amountIn * 997;
        amountOut = (amountInWithFee * rOut) / (rIn * 1000 + amountInWithFee);
    }

    /// Выполняет swap через router от имени victim (caller должен иметь tokenIn)
    function victimSwap(
        address victim,
        address tokenIn,
        address tokenOut,
        uint256 amountIn,
        uint256 deadline
    ) internal returns (uint256 amountOut) {
        address[] memory path = new address[](2);
        path[0] = tokenIn;
        path[1] = tokenOut;

        IERC20(tokenIn).approve(UNIV2_ROUTER, amountIn);
        uint256[] memory amounts = IUniswapV2Router02(UNIV2_ROUTER).swapExactTokensForTokens(
            amountIn, 0, path, victim, deadline
        );
        amountOut = amounts[amounts.length - 1];
    }

    /// Возвращает адрес пары
    function getPair(address tokenA, address tokenB) internal view returns (address) {
        return IUniswapV2Factory(UNIV2_FACTORY).getPair(tokenA, tokenB);
    }
}
