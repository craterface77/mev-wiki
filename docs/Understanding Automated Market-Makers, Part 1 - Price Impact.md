Every day, [thousands of people use a decentralized exchange (DEX) for the first time](https://www.theblockcrypto.com/data/decentralized-finance/dex-non-custodial). However, the **idiosyncrasies of a public blockchain** routinely catch newcomers off-guard, even those familiar with trading on more traditional venues. As a result, traders bleed money to arbitrageurs and frontrunners, leading to worse-than-necessary execution.

At a high level, we can break down the costs of each trade into several parts:

1. **Price impact**
2. **Broker or trading fees**
3. **Slippage**
4. **Transaction fees of the underlying blockchain**

This article on automated market-makers (AMMs) will serve as an intro to the series and discuss the first and most crucial cost: price impact. You will learn

- how AMMs like Uniswap v2, Sushiswap, and Balancer[1](https://research.paradigm.xyz/amm-price-impact#fn:1) determine the prices they quote; and
- how to minimize the price impact of your trade using a few simple strategies.

# What are liquidity pools?

Most DEXs consist of many **liquidity pools** that represent different trading pairs, like ETH/WBTC. Instead of matching buyers and sellers in an orderbook, these liquidity pools act as an **automated market maker**.

A liquidity pool is a smart contract that holds reserves of two or more tokens and allows anyone to deposit and withdraw funds from them, but only according to very specific rules.

One such rule is the **constant product formula x * y = k,** where x and y are the reserves of two tokens, A and B. In order to withdraw some amount of token A, one must deposit a proportional amount of token B to maintain the constant k before fees.[2](https://research.paradigm.xyz/amm-price-impact#fn:2)

# How does an AMM determine its price?

From the constant product formula it follows that the price of that token A is simply _price_token_A = reserve_token_B / reserve token_A_.

![](https://cdn.sanity.io/images/dgybcd83/production/77445c2f27457e58406bc250c491481273cdccdb-1002x615.png?auto=format&q=75&url=https://cdn.sanity.io/images/dgybcd83/production/77445c2f27457e58406bc250c491481273cdccdb-1002x615.png&w=800)

_Chart 1: Different AMM formulas result in different pricing curves. When a hypothetical Uniswap v2 liquidity pool has 15 Y-tokens, it will only pay 0.1 X-tokens for the marginal Y-token. But when it has only 2.5 Y-token, it will pay 4.0 X-tokens. Other pricing curves are designed to concentrate more liquidity around a certain price (e.g. 1.0 for stablecoins). Source: Curve Whitepaper_

To look at a real-world example, at the time of writing there are 2,700 WBTC and 86,000 ETH in Uniswap’s [ETH/WBTC pool](https://info.uniswap.org/pair/0xbb2b8038a1640196fbe3e38816f3e67cba72d940). This reserve ratio implies that ETH’s **market price** at the time of writing is 2,700 / 86,000 = 0.0314 WBTC.

Crucially, **the AMM does not update this price as other markets move around it**. The market price only moves as the reserve ratio of the tokens in the pool changes, which happens when someone trades against it.

To explore an example, what happens if the price on Binance falls to 0.0310 WBTC? That implies Uniswap LPs are currently buying ETH at a premium, creating an **arbitrage opportunity**. As a result, **arbitrageurs** buy the “cheap” ETH on Binance and sell it on Uniswap for an immediate profit. They keep doing this until the next unit of Uniswap ETH only pays 0.0310 WBTC—same as on Binance—and they can no longer profit by selling more. In our example above, this point is reached after selling 550 ETH to the pool for 17.2 WBTC (ignoring fees and gas for simplicity).

As a result, even though AMMs don’t update their prices based on incoming real-world information, traders can still expect the price quoted by an AMM to [closely track the global market price because of continuous arbitrage](https://web.stanford.edu/~guillean/papers/uniswap_analysis.pdf).

# What is Price Impact?

While we learned how to compute the current market price from the ratio of two token reserves, this market price only shows the **price the AMM wants for the marginal token**. However, in practice, a trader will often buy or sell many tokens at once, with every token costing more than the previous one.

This **difference between the current market price and the expected fill price is called [price impact](https://uniswap.org/docs/v2/protocol-overview/glossary/#price-impact)**.

Price impact is a function of

- the size of your trade relative to the size of the liquidity pool; as well as
- the trading rule being used (e.g. constant product formula).

![](https://cdn.sanity.io/images/dgybcd83/production/0f3d10e61d6f124b22910d32a40825f7ec53bf8b-600x371.png?auto=format&q=75&url=https://cdn.sanity.io/images/dgybcd83/production/0f3d10e61d6f124b22910d32a40825f7ec53bf8b-600x371.png&w=800)

_Price impact of different trade sizes_

Chart 2: Comparing the average fill price (left y-axis) and price impact (right y-axis) for different order sizes (x-axis). Both factors increase with order size. The larger the order relative to the pool, the further above the market price will the trade get filled.

![](https://cdn.sanity.io/images/dgybcd83/production/f0bec2ec8e94949baa128afdedee77c737c227de-600x371.png?auto=format&q=75&url=https://cdn.sanity.io/images/dgybcd83/production/f0bec2ec8e94949baa128afdedee77c737c227de-600x371.png&w=800)

_Price impact of sell order on different pool sizes_

_Chart 3: Comparing the average fill price (left y-axis) and price impact (right y-axis) of a 10 WBTC sell order on different pool sizes on Uniswap V2 (x-axis). The pool size is the total value of the pool including the reserves of both assets. The orders represent 0.19%, 1.85%, and 18.52% of the pool respectively. So a good rule of thumb is that **the price impact of your order is about twice the size of your order relative to the pool**._

# How can Price Impact be minimized?

As we alluded earlier, price impact can represent a large share of a trade’s overall execution cost. Here are some simple strategies to minimize it:

- **Find the deepest market:** So far, we established that price impact is a function of trade size relative to the size of the pool or market. It follows then that we want to **find the pool that has the most liquidity in the price range we care about**, which is where we will get filled closest to the market price. A token’s [depth table on Coingecko](https://www.coingecko.com/en/coins/uniswap#markets) is a good starting point to analyze this.

![](https://cdn.sanity.io/images/dgybcd83/production/7ece7d9006bddc8a419a4f77874fc21628b73966-798x372.png?auto=format&q=75&url=https://cdn.sanity.io/images/dgybcd83/production/7ece7d9006bddc8a419a4f77874fc21628b73966-798x372.png&w=800)

_Comparing market depth_

_Chart 4: Trading pairs for UNI, sorted by liquidity within 2% of the market price. Note the difference in spread between Uniswap and Bitfinex. Source: Coingecko_

- **Look outside of DeFi:** While this is a post on AMMs, we won’t pretend you can always get the best execution on-chain. In fact, since the AMMs discussed spread their liquidity across a continuous range of prices, they often have little liquidity concentrated around the current market price. This is a known problem that many DEXs are trying to solve, for example, Uniswap v3 will [allow market makers to place their liquidity concentrated around the current market price](https://uniswap.org/blog/uniswap-v3/), making prices more competitive to CEXs as a resultWhen a trade moves the price on a DEX and the same token trades in other markets, an arbitrage opportunity is created. As discussed, arbitrageurs will “backrun” the trade (i.e., insert their own transaction immediately after) and move the price back to the global market price. It’s easy to see why **the existence of such an arbitrage is itself proof of an execution mistake**, as the trader is donating capital to the arbitrageur. This begs the question: should you execute an on-chain trade with more than 2-3% price impact if other markets exist?
- **Mind the trading fee**: AMMs have a trading fee of 0.30%, which translates to a spread of 0.6% between the best buy order and the best sell order. Within this range, the AMM does not quote any prices. In other words, **even the most liquid AMM trade has an implicit 0.3% price impact**. Minimizing the impact of fees is extremely important, especially for trades that would have very little price impact on a CEX, so a CEX might be the better execution venue outright. (For comparison, the same trade would have a [0.10%](https://www.binance.com/en/fee/trading) fee on Binance and [0.07%](https://help.ftx.com/hc/en-us/articles/360024479432-Fees) on FTX.)That said, there are other reasons to pay more for DEX access, including retaining full custody or avoiding an onboarding, KYC, or deposit process. However, even in those cases, traders should be aware that their higher execution prices include an implicit **decentralization or instant liquidity premium**.
- **Spread out trades**: First, it’s possible to split one trade into several smaller trades over time. This is especially relevant for traders who prefer to trade on a DEX in spite of other liquid markets existing outside DeFi. In that case, you can e.g., buy in 20% increments and let arbitrageurs revert the price after every trade. These five orders together will cause lower price impact than a single large order, but with the added tradeoff of higher gas costs and execution time. The larger the trade, the better this strategy is, as the fixed cost of gas decreases relative to the benefit of marginally better execution. This strategy also works when assets are mean reverting, e.g. stablecoins.
- **The direct route is not always the cheapest**: Not every trade has a direct token pair, and even if it does, it may be cheaper to use a **bridge currency** instead. For example, while tokens A:B might have a direct pair, it is often cheaper to trade A → ETH → B, if these pairs are sufficiently more liquid. Aggregators are very useful in that regard, even if you just use them to look at their route suggestions.
- **Use DEX aggregators**: Finally, you can use a DEX aggregator like 1inch, Matcha or Paraswap. Aggregators are the DeFi equivalent of [smart order routing](https://en.wikipedia.org/wiki/Smart_order_routing) and work because AMMs will sell the first token at a cheaper price than the tenth token. Whenever a token trades in multiple pools, aggregators will buy the token across all pools in order to minimize price impact on each one of them. Instead of spreading the trade over time in a single market, this order executes at once, spread over many possible markets. Aggregators also command substantially higher gas costs than a single trade, similar to splitting trades manually.

![](https://cdn.sanity.io/images/dgybcd83/production/1160e8a6a19a8f32eb0f07ece093e9bf8a88e5d1-1200x742.png?auto=format&q=75&url=https://cdn.sanity.io/images/dgybcd83/production/1160e8a6a19a8f32eb0f07ece093e9bf8a88e5d1-1200x742.png&w=800)

Chart 5: The optimal strategy to buy 10 ($3,200), 50 ($16,000), 100 ($32,000), and 200 AAVE ($64,000) with ETH. The larger the trade, the more exchanges are added to the route to avoid moving any individual pool too much. Source: 1inch  

  

# Outlook

In the second part this series, we will focus entirely on **slippage**. Almost all AMM trades are subject to frontrunning, causing them to be **filled at the maximum slippage the trader is willing to accept**. This is a unique “feature” of trading on a public blockchain that cannot be avoided with how decentralized exchanges work today. The cost can only be shifted around, leading us to formulate “The Sandwich Trilemma”.

_Credits: Thanks for valuable discussions and reviews to [EvanSS](https://twitter.com/Evan_ss6), [Georgios Konstantopoulos](https://twitter.com/gakonst), [Dave White](https://twitter.com/_Dave__White_), [Dan Robinson](https://twitter.com/danrobinson), [Arjun Balaji](https://twitter.com/arjunblj), and [raul](https://twitter.com/raulGpoker)_

_Disclaimer: This post is for general information purposes only. It does not constitute investment advice or a recommendation or solicitation to buy or sell any investment and should not be used in the evaluation of the merits of making any investment decision. It should not be relied upon for accounting, legal or tax advice or investment recommendations. This post reflects the current opinions of the authors and is not made on behalf of Paradigm or its affiliates and does not necessarily reflect the opinions of Paradigm, its affiliates or individuals associated with Paradigm. The opinions reflected herein are subject to change without being updated._

## Notes

1. Other forms of DEXs based on a central-limit order book (such as Serum) or batch auctions (such as Gnosis) are not in scope for this article. [↩](https://research.paradigm.xyz/amm-price-impact#fnref:1)
2. Fees slightly increase the k every trade [↩](https://research.paradigm.xyz/amm-price-impact#fnref:2)

### Written by

[Hasu](https://x.com/hasufl)

