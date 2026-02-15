## Introduction

As you dive into building your MEV bot, you quickly realize that while **executing** your logic is crucial, an often underestimated aspect of the process is the **search** for MEV opportunities. Today, I want to discuss how we can effectively search for these opportunities, which can occur in two main regions:

1. **Mempool data:** pending transactions
2. **Transaction/Event data:** confirmed transactions

When it comes to MEV, the term often brings to mind the analysis of mempool data, specifically pending transactions. You may have already come across concepts like front-running, back-running, and sandwiching bots that leverage pending transactions to profit from them. These bots scour the mempool, which contains transactions that are yet to be included in a block, and attempt to identify potential transactions that can benefit them in some way.

The idea behind these bots is to analyze the pending transactions in the mempool and simulate various scenarios to determine if any of these transactions can be exploited for profit. By carefully examining the characteristics of these transactions, such as their content, gas fees, and intended targets, these bots can assess the potential outcomes and make informed decisions on whether to pursue them.

However, today I want to talk about a simpler searching/simulation method that every on-chain trader should understand and use in their strategy development process. **This method uses data that have already been confirmed on the blockchain.** Though state transitions have occurred and finalized on the blockchain, there may still be discrepancies among various DEX protocols, and I’d like to simulate multiple swap paths across these protocols and see if my path is profitable or not.

## Why do we need a simulation engine?

You may be wondering why we need a simulation engine at all. If we are talking about extremely simple strategies that involve interacting with a single DEX protocol, then we might not need it. But we are talking about multiple DEXs on a variety of blockchain ecosystems. So yes, to accurately figure out if your trade will go through successfully, and to simulate how much profit you are expected to earn, every MEV bot developer will have to build out their own simulation engines.

As a matter of fact, skimming through Github repos related to MEV bots will give you the whole codebase for bots that can execute their trades using private relays like Flashbots, but what they won’t give you is the searching/simulation engine.

Below is a full implementation of a sandwiching bot written in JS by **libevm**:

[

## GitHub - libevm/subway: A practical example on how to perform sandwich attacks on Ethereum

### A practical example on how to perform sandwich attacks on Ethereum - GitHub - libevm/subway: A practical example on how…

github.com



](https://github.com/libevm/subway?source=post_page-----c9c0420d2e1---------------------------------------)

Another is one by **Flashbots**:

[

## GitHub - flashbots/simple-arbitrage: Example arbitrage bot using Flashbots

### Example arbitrage bot using Flashbots. Contribute to flashbots/simple-arbitrage development by creating an account on…

github.com



](https://github.com/flashbots/simple-arbitrage?source=post_page-----c9c0420d2e1---------------------------------------)

_I’ll have the chance to review these codes in a later blog post when I’m touching on the execution side of MEV bots._

We now understand that building our own searching bot and simulation engine is important. But where do we begin?

**1. Search**

**2. Simulate**

Search and simulate. These are both very important, but they both require a different set of knowledge, so today, I’ll just focus on the **simulation engine** part.

## Let’s start building right away

The best way to start learning is by doing, especially in the field of blockchains — and in trading!

**What are we building today?**

I want to perform multi-hop swaps (_a typical n-way arbitrage like triangular arbitrages_) across a number of DEXs using a single chain. For instance, my swaps could occur in Curve Finance, Uniswap V2, Uniswap V3, and any other number of DEXs that you want to include.

**What are we simulating?**

I want to figure out if the n-way path I found will be profitable if I sent a transaction performing the swaps.

This could be done in one of two ways:

1. **By coding up your own simulator** that has all the price impact functions implemented. (Why price impact is critical is laid out here: [https://www.paradigm.xyz/2021/04/understanding-automated-market-makers-part-1-price-impact](https://www.paradigm.xyz/2021/04/understanding-automated-market-makers-part-1-price-impact))
2. **By using smart contracts** to simulate price impacts.

Today, I am going to use the second approach, because the first approach is very time consuming. With the first approach, you’ll have to understand how your swap/trades will affect the pair prices on different DEX protocols by reading their docs and contracts thoroughly. The difficult part about this is that the AMM formulas these DEXs use all differ from one another.

But with the second approach, you won’t have to understand those formulas. All you need is a bit of browsing through, and figuring out what functions DEXs use to simulate their swaps. This information is often public and is easily found with some familiarity with Solidity. ==Also, calling smart contract functions is free of charge if you are not changing the blockchain state - other than the contract creation fee.==

## Project setup

I’ll be using Foundry to write out my smart contract to simulate the potentially profitable swap paths.

Foundry uses Rust so you will need to have Rust/Cargo installed. Run the below command to install Foundryup:

curl -L https://foundry.paradigm.xyz | bash

Now run:

foundryup

This will install all the commands you need to start building with Foundry.

Now that you have the dependencies installed, you can initialize your Foundry project:

forge init swap-simulator-v1  
cd swap-simulator-v1 && forge build  
forge install OpenZeppelin/openzeppelin-contracts

In the _src_ directory, create a new Solidity file called **“SimulatorV1.sol”.**

// SPDX-License-Identifier: MIT  
pragma solidity ^0.8.9;  
  
import "openzeppelin-contracts/contracts/utils/math/SafeMath.sol";  
  
contract SimulatorV1 {  
    using SafeMath for uint256;  
  
    // Polygon network addresses  
    address public UNISWAP_V2_FACTORY = 0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32;  
    address public UNISWAP_V3_QUOTER2 = 0x61fFE014bA17989E743c5F6cB21bF9697530B21e;  
  
    struct SwapParams {  
        uint8 protocol;   // 0 (UniswapV2), 1 (UniswapV3), 2 (Curve Finance)  
        address pool;     // used in Curve Finance  
        address tokenIn;  
        address tokenOut;  
        uint24 fee;       // only used in Uniswap V3  
        uint256 amount;   // amount in (1 USDC = 1,000,000 / 1 MATIC = 1 * 10 ** 18)  
    }  
  
    constructor() {}  
  
    function simulateSwapIn(SwapParams[] memory paramsArray) public returns (uint256) {  
  
    }  
  
    function simulateUniswapV2SwapIn(SwapParams memory params) public returns (uint256 amountOut) {  
  
    }  
  
    function simulateUniswapV3SwapIn(SwapParams memory params) public returns (uint256 amountOut) {  
  
    }  
  
    function simulateCurveSwapIn(SwapParams memory params) public returns (uint256 amountOut) {  
  
    }  
}

This is the basic structure of our simulator. We will be using Polygon, because the gas fee there is very cheap and, thus, is a good place to test out your code.

The code is pretty self-explanatory, as we can see that we will be calling “simulateSwapIn” function by sending in an array of SwapParams which is a struct.

We will now build the function:

function simulateSwapIn(SwapParams[] memory paramsArray) public returns (uint256) {  
    uint256 amountOut = 0;  
    uint256 paramsArrayLength = paramsArray.length;  
  
    for (uint256 i; i < paramsArrayLength;) {  
        SwapParams memory params = paramsArray[i];  
  
        if (amountOut == 0) {  
            amountOut = params.amount;  
        } else {  
            params.amount = amountOut;  
        }  
  
        if (params.protocol == 0) {  
            amountOut = simulateUniswapV2SwapIn(params);  
        } else if (params.protocol == 1) {  
            amountOut = simulateUniswapV3SwapIn(params);  
        } else if (params.protocol == 2) {  
            amountOut = simulateCurveSwapIn(params);  
        }  
  
        unchecked {  
            i++;  
        }  
    }  
  
    return amountOut;  
}

Insert this function definition into the empty “simulateSwapIn” block above. Don’t worry about what it does yet. We will get into that soon.

Before we look at this function though, we need to understand how DEXs let you simulate your trades with functions like:

- **getAmountOut (UniswapV2)**
- **quoteExactInputSingle (UniswapV3)**
- **get_dy (Curve Finance)**

### First, UniswapV2.

UniswapV2 is the easiest of all. And since so many DEXs are UniswapV2 forks, this method will apply to others as well.

If you go here:

[

## v2-periphery/contracts/libraries/UniswapV2Library.sol at master · Uniswap/v2-periphery

### 🎚 Peripheral smart contracts for interacting with Uniswap V2 - v2-periphery/contracts/libraries/UniswapV2Library.sol…

github.com



](https://github.com/Uniswap/v2-periphery/blob/master/contracts/libraries/UniswapV2Library.sol?source=post_page-----c9c0420d2e1---------------------------------------)

You will see a function like this:

// given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset  
function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) internal pure returns (uint amountOut) {  
    require(amountIn > 0, 'UniswapV2Library: INSUFFICIENT_INPUT_AMOUNT');  
    require(reserveIn > 0 && reserveOut > 0, 'UniswapV2Library: INSUFFICIENT_LIQUIDITY');  
    uint amountInWithFee = amountIn.mul(997);  
    uint numerator = amountInWithFee.mul(reserveOut);  
    uint denominator = reserveIn.mul(1000).add(amountInWithFee);  
    amountOut = numerator / denominator;  
}

With this function, you can simulate how much tokens you are going to get out if you inputted “amountIn” into this UniswapV2 pool.

[](https://medium.com/blog/newsletter?source=promotion_paragraph---post_body_banner_beneficial_intelligence_nl--c9c0420d2e1---------------------------------------)

To use this, create a new directory in **_src_** called **_protocols_**, then **_uniswap_** within **_protocols_**. It will look like this **_src/protocols/uniswap_**:

![](https://miro.medium.com/v2/resize:fit:1324/1*4JBdceTUp2WdxFdJUFvSEw.png)

I’ve already added all the Solidity files I need in my **_src_** directory. You can do the same. Now copy and paste UniswapV2Library.sol file into your **src/protocols/uniswap/UniswapV2Library.sol**. There’s a catch though. The Solidity compiler version used for UniswapV2 doesn’t match that of the more modern ones. So hop on over to my Github and copy, paste the code from there. That should work then.

This is the repo:

[

## GitHub - solidquant/swap-simulator-v1: Simulating multiple swaps across multiple DEXs using…

### Simulating multiple swaps across multiple DEXs using Solidity - GitHub - solidquant/swap-simulator-v1: Simulating…

github.com



](https://github.com/solidquant/swap-simulator-v1?source=post_page-----c9c0420d2e1---------------------------------------)

After you are done creating the interfaces/library files within your **_protocols_** directory, you are now ready to understand how we simulate swaps in UniswapV2.

Let’s come back to the SimulatorV1 code:

// SPDX-License-Identifier: MIT  
pragma solidity ^0.8.9;  
  
import "openzeppelin-contracts/contracts/utils/math/SafeMath.sol";  
  
// do all the imports  
import "./protocols/uniswap/UniswapV2Library.sol";  
import "./protocols/uniswap/IQuoterV2.sol";  
import "./protocols/curve/ICurvePool.sol";  
  
contract SimulatorV1 {  
    using SafeMath for uint256;  
  
    // Polygon network addresses  
    address public UNISWAP_V2_FACTORY = 0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32;  
    address public UNISWAP_V3_QUOTER2 = 0x61fFE014bA17989E743c5F6cB21bF9697530B21e;  
  
    struct SwapParams {  
        uint8 protocol;  
        address pool;  
        address tokenIn;  
        address tokenOut;  
        uint24 fee;  
        uint256 amount;  
  
    constructor() {}  
  
    // other code here  
  
    function simulateUniswapV2SwapIn(SwapParams memory params) public returns (uint256 amountOut) {  
        (uint reserveIn, uint reserveOut) = UniswapV2Library.getReserves(  
            UNISWAP_V2_FACTORY,  
            params.tokenIn,  
            params.tokenOut  
        );  
        amountOut = UniswapV2Library.getAmountOut(  
            params.amount,  
            reserveIn,  
            reserveOut  
        );  
    }  
  
    // other code here  
}

Using UniswapV2Library, we get the reserves from the exchange pair consisting of tokenIn and tokenOut. These will be addresses such as:

- **USDC:** 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 ([https://polygonscan.com/address/0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174](https://polygonscan.com/address/0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174))
- **USDT:** 0xc2132D05D31c914a87C6611C10748AEb04B58e8F ([https://polygonscan.com/address/0xc2132D05D31c914a87C6611C10748AEb04B58e8F](https://polygonscan.com/address/0xc2132D05D31c914a87C6611C10748AEb04B58e8F))

With those reserves, it will then call “getAmountOut” and get the swap result of trading “amount”. We return this value.

### Second, UniswapV3.

UniswapV3 is a bit more complicated, but don’t get too scared. The documentations tell you a great deal. Expecially right here:

[

## QuoterV2 | Uniswap

### Allows getting the expected amount out or amount in for a given swap without executing the swap

docs.uniswap.org



](https://docs.uniswap.org/contracts/v3/reference/periphery/lens/QuoterV2?source=post_page-----c9c0420d2e1---------------------------------------)

Using Quoter2, you can use “quoteExactInputSingle” to simulate a single swap from V3 pools. Again, to achieve this, go to my Github and copy, paste IQuoterV2.sol in **_src/protocols/uniswap/IQuoterV2.sol_**.

Now to the SimulatorV1.sol file again:

// imports...  
  
contract SimulatorV1 {  
  
    // other code here  
  
    function simulateUniswapV3SwapIn(SwapParams memory params) public returns (uint256 amountOut) {  
        IQuoterV2 quoter = IQuoterV2(UNISWAP_V3_QUOTER2);  
        IQuoterV2.QuoteExactInputSingleParams memory quoterParams;  
        quoterParams.tokenIn = params.tokenIn;  
        quoterParams.tokenOut = params.tokenOut;  
        quoterParams.amountIn = params.amount;  
        quoterParams.fee = params.fee;  
        quoterParams.sqrtPriceLimitX96 = 0;  
        (amountOut,,,) = quoter.quoteExactInputSingle(quoterParams);  
    }  
  
    // other code here  
}

Since QuoterV2 is a contract that is actually deployed to the network (as can be seen from here: [https://docs.uniswap.org/contracts/v3/reference/deployments](https://docs.uniswap.org/contracts/v3/reference/deployments)), you will need to wrap the address for the QuoterV2 with an interface of IQuoterV2, and create **QuoteExactInputSingleParams** input struct to call the target function.

### Third, Curve Finance.

Curve is a bit trickier, since it’s so different from other Uniswap fork DEXs. But this project actually has the interface made out for us:

[

## GitHub - studydefi/money-legos: 💰One stop shop for Ethereum ABIs, addresses, and Solidity…

### 💰One stop shop for Ethereum ABIs, addresses, and Solidity interfaces! - GitHub - studydefi/money-legos: 💰One stop…

github.com



](https://github.com/studydefi/money-legos?source=post_page-----c9c0420d2e1---------------------------------------)

I copy, pasted the interface for Curve Finance pools from here. After I did that, I checked to see if it was up to date. And I checked that at least with 3pool I was interested in using from Curve Finance, the interfaces matched up:

[

## curve-contract/contracts/pools/3pool/StableSwap3Pool.vy at master · curvefi/curve-contract

### Vyper contracts used in Curve.fi exchange pools. Contribute to curvefi/curve-contract development by creating an…

github.com



](https://github.com/curvefi/curve-contract/blob/master/contracts/pools/3pool/StableSwap3Pool.vy?source=post_page-----c9c0420d2e1---------------------------------------)

After this has been setup, let’s go to our SimulatorV1.sol file again:

// imports...  
  
contract SimulatorV1 {  
  
    // other code here  
  
    function simulateCurveSwapIn(SwapParams memory params) public returns (uint256 amountOut) {  
        ICurvePool pool = ICurvePool(params.pool);  
  
        int128 i = 0;  
        int128 j = 0;  
  
        int128 coinIdx = 0;  
  
        while (i == j) {  
            address coin = pool.coins(coinIdx);  
  
            if (coin == params.tokenIn) {  
                i = coinIdx;  
            } else if (coin == params.tokenOut) {  
                j = coinIdx;  
            }  
  
            if (i != j) {  
                break;  
            }  
  
            unchecked {  
                coinIdx++;  
            }  
        }  
  
        amountOut = ICurvePool(params.pool).get_dy(  
            i,  
            j,  
            params.amount  
        );  
    }  
}

This looks a bit more difficult, because Curve doesn’t store information about token 0, token 1. This is because Curve pools can take more than 2 tokens as pairs. And with 3pool, there are 3 stablecoins in the pool. Others can have more as well.

So we run a while loop in Solidity and try and match tokens with the index number used from that pool’s contract.

After we figure out the coin index of our tokenIn and tokenOut, we call “get_dy” function to simulate the stableswap from Curve Finance. We return that value as well.

## The simulateSwapIn function

Now we can understand the “simulateSwapIn” function, which we will look at again:

function simulateSwapIn(SwapParams[] memory paramsArray) public returns (uint256) {  
    // init the resulting value to 0  
    uint256 amountOut = 0;  
    uint256 paramsArrayLength = paramsArray.length;  
  
    // loop through each values in paramsArray one by one  
    for (uint256 i; i < paramsArrayLength; ) {  
        SwapParams memory params = paramsArray[i];  
  
        // if no swaps have been simulated yet, set amountOut to be  
        // the initial amount in value from params struct  
        if (amountOut == 0) {  
            amountOut = params.amount;  
        } else {  
            // if amountOut isn't 0, meaning a swap path has been simulated  
            // at least once, use that output to be the "amount"  
            params.amount = amountOut;  
        }  
  
        if (params.protocol == 0) {  
            amountOut = simulateUniswapV2SwapIn(params);  
        } else if (params.protocol == 1) {  
            amountOut = simulateUniswapV3SwapIn(params);  
        } else if (params.protocol == 2) {  
            amountOut = simulateCurveSwapIn(params);  
        }  
  
        // don't worry about this part  
        // it simply increments i by 1  
        // this code is referenced from: https://github.com/Uniswap/universal-router/blob/main/contracts/UniversalRouter.sol  
        unchecked {  
            i++;  
        }  
    }  
  
    return amountOut;  
}

The code above makes so much more sense now.

If we are done writing the Simulator code, we should deploy it to the production network to give it a test — be it mainnet, testnet. I’ll deploy it to the mainnet right away.

Using Foundry, you can deploy this smart contract very easily:

forge create --rpc-url <rpc-url> --private-key <private-key> src/SimulatorV1.sol:SimulatorV1

Calling this command with your RPC URL (I used Alchemy) and a private key would deploy your contract right away after auto compiling your Solidity code. For more information on this refer to the below:

[

## Foundry Book

### A book on all things Foundry

book.getfoundry.sh



](https://book.getfoundry.sh/forge/deploying?source=post_page-----c9c0420d2e1---------------------------------------)

The output of the above command is as follows:

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/1*WtzSUHVq4qGBsevH5RYPPg.png)

_I put the ( — legacy) flag there, because I deployed to Polygon Mainnet._

The SimulatorV1 contract address is: **0x37384C5D679aeCa03D211833711C277Da470C670**

Now that we’ve deployed our contract, let’s try calling the simulation function using Javascript. It should work with other languages as well, because now your simulation function is live on the blockchain, any web3 libraries such as **web3.js, ethers.js, web3.py, web3.rs, web3.go** should work just the same.

I’ll use ethers.js to test out my simulation function.

Within the swap-simulator-v1 directory, create a Node.js project:

npm init  
npm install --save-dev ethers@5.7.2 dotenv

_This project should work with all ethers versions, but I just stuck to 5.7.2, because Flashbots doesn’t work with versions above this, and I want to use Flashbots for this project later in the future._

Next, type in the JS script:

const { ethers } = require("ethers");  
  
require("dotenv").config();  
  
const SimulatorV1ABI = require("./out/SimulatorV1.sol/SimulatorV1.json").abi;  
  
// DO NOT USE REAL PRIVATE KEY  
const provider = new ethers.providers.JsonRpcProvider(process.env.ALCHEMY_URL);  
const signer = new ethers.Wallet(process.env.TEST_PRIVATE_KEY, provider);  
  
const SimulatorV1Address = "0x37384C5D679aeCa03D211833711C277Da470C670";  
  
const contract = new ethers.Contract(  
  SimulatorV1Address,  
  SimulatorV1ABI,  
  signer  
);  
  
(async () => {  
  const swapParam1 = {  
    protocol: 0,  
    pool: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", // random address  
    tokenIn: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",  
    tokenOut: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",  
    fee: 0,  
    amount: ethers.utils.parseUnits("1", 6),  
  };  
  
  const swapParam2 = {  
    protocol: 1,  
    pool: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", // random address  
    tokenIn: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",  
    tokenOut: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",  
    fee: 500,  
    amount: 0, // no need  
  };  
  
  const swapParam3 = {  
    protocol: 2,  
    pool: "0x445FE580eF8d70FF569aB36e80c647af338db351", // real Curve.fi pool address  
    tokenIn: "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",  
    tokenOut: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",  
    fee: 0,  
    amount: 0, // no need  
  };  
  
  const swapParams = [swapParam1, swapParam2, swapParam3];  
  
  const amountOut = await contract.callStatic.simulateSwapIn(swapParams);  
  console.log(amountOut.toString());  
})();

Run it as is, and it should work on mainnet, because it’s deployed there.

I am trying to simulate swap paths using 1 USDC:

**(UniswapV2) USDC → USDT**

**(UniswapV3) USDT → USDC**

**(Curve Finance) USDC → USDT**

The end result will be: 996819 (= 0.996819 USDT).

A pretty useless path to simulate, but it demonstrates the purpose well.

## Conclusion

This post ended up being pretty long. So for those of you that just like to dive into code right away can go refer to my Github repo at:

[

## GitHub - solidquant/swap-simulator-v1: Simulating multiple swaps across multiple DEXs using…

### Simulating multiple swaps across multiple DEXs using Solidity - GitHub - solidquant/swap-simulator-v1: Simulating…

github.com



](https://github.com/solidquant/swap-simulator-v1?source=post_page-----c9c0420d2e1---------------------------------------)

Also, for people that are just getting started with the MEV bot building journey, you guys are not alone. I got started a couple of weeks ago, with a little bit of background knowledge/experience in CeFi trading, and I feel like talking to people is the surest way to solve a lot of problems here. I also suffer from lack of content in this domain, but it is quite understandable.

So follow me on Twitter, we can talk about related topics in more depth there! See you in the next post :)

[

  




](https://twitter.com/solidquant?source=post_page-----c9c0420d2e1---------------------------------------)