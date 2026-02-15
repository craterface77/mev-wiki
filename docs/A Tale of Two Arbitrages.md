## TL;DR

- For the same opportunity, more value can be extracted via CeFi-DeFi (EV_signal) than atomic (EV_ordering) arbitrage. Whether an arbitrage is executed by EV_signal or EV_ordering style strategy depends on the cost of risk-taking.
- In the first quarter of 2023, 60% of the arbitrage opportunity was captured by CeFi-DeFi strategies.
- For highly liquid tokens (i.e., where the cost of risk-taking is low), EV_signal dominates, whereas for low liquidity tokens EV_ordering dominates.
- Atomicity is less important than liquidity in many situations. The future of cross-chain arbitrage will not be risk-free atomic execution (e.g., Chain <> Chain) but economically efficient statistical execution (e.g., Chain <> CEX).

## Introduction

This work is an extension of “[A new game in town](https://frontier.tech/a-new-game-in-town),” where we introduce EV_signal and EV_ordering. The difference between these two Extractable Values (EV) is **information** (aka alpha). EV_signal requires informational advantage to capture value, whereas EV_ordering does not. In this article, we introduce the notion of **risk** in the MEV equation. Theoretically, risk-taking traders can extract more value from an opportunity compared to risk-free traders.

We examine the theory and market structure behind EV_signal and EV_ordering with respect to arbitrage opportunities. More specifically, we study atomic arbitrage and CeFi-DeFi arbitrage — subsets of EV_ordering and EV_signal, respectively — and demonstrate that while atomic arbitrage is risk-free, CeFi-DeFi arbitrage requires risk-taking. We compare them in terms of their theoretical framework and execution, thereafter using the findings to make predictions for the future of on-chain trading.

This article is divided into four main sections:

- First, we define atomic and CeFi-DeFi arbitrage and explore the conditions in which they get executed on-chain.
- Second, we compare them theoretically and investigate under which conditions one might prevail over the other.
- Third, we measure the arbitrage activity on-chain and empirically compare their market size.
- Finally, we take these learnings and make predictions for the future.

## Types of Arbitrage

Arbitrage refers to trading a price discrepancy between different trading venues such that the prices are brought into equilibrium, and a profit is realized. In the simplest terms, it involves buying the asset at the lower-priced venue and selling it at the higher-priced venue, or vice-versa. In crypto, there are thousands of tokens and hundreds of trading venues (both on-chain and off-chain). Any price dislocation between them can create an arbitrage opportunity.

### Atomic Arbitrage

Atomic arbitrage was one of the earliest MEV opportunities we saw in the wild. The simplest example of atomic arbitrage is when a trading pair is listed on multiple DEXes with different prices. The below image depicts how atomic arbitrage strategies trade on two on-chain DEXes until the price is in equilibrium.

![(Left) Prices on the three exchanges Binance, Uniswap, and SushiSwap are in equilibria. (Center) A user order significantly moves the price on Uniswap. (Right) An atomic arbitrage (back-run) brings the DEX prices back into equilibrium.](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/e8b0d994-9269-4b66-a1c7-5b270c0ec615/Screenshot_2023-05-16_at_12.05.10/w=1920,quality=90,fit=scale-down)

(Left) Prices on the three exchanges Binance, Uniswap, and SushiSwap are in equilibria. (Center) A user order significantly moves the price on Uniswap. (Right) An atomic arbitrage (back-run) brings the DEX prices back into equilibrium.

**Executing Atomic Arbitrage**

An atomic arbitrage is carried out in a singular, isolated event—hence the name. Either all legs of the trade are executed, or none are. The arbitrage happens instantaneously, and the trader holds no inventory between the two legs of the trade, making this strategy risk-free with respect to hedging the inventory. Moreover, off-chain searcher infrastructure (Flashbots Auction under PoW and block builder private RPCs under PoS) guarantees revert protection; that is, failed transactions do not land on-chain, posing zero cost to the trader. Due to the above two reasons, this strategy is theoretically riskless and has low barriers to entry.

As these trades are riskless (on mainnet) and have low barriers to entry, they are highly competitive to execute. Under MEV-Boost, the searcher who tips the block builder the most gets included in the block builder’s submission, and the block builder who tips the validator the most wins the block. Currently, 91-99% of the extractable value gets sent to validators by the winning searchers.

Atomic arbitrage opportunities become more complicated as we add more DEXes and tokens into the picture. For example, routes can involve more than two trading pairs or two tokens. But the core idea is the same: prices across multiple DEX venues are dislocated, exposing an opportunity for arbitrageurs to atomically and profitably trade until the prices are in line.

### CeFi-DeFi Arbitrage

CeFi-DeFi arbitrage opportunities arise when the price of an asset on an on-chain venue deviates from its **fair value**. In the simplest terms, fair value is the current best estimate of the valuation of an asset. The trading venue where prices are closest to the fair value is called the **venue of price discovery** (this venue regularly changes). In crypto, it is possible to estimate a fair value using prices from the most liquid or highest-volume trading venues; these are centralized exchanges (CeFi); hence this strategy is called CeFi-DeFi. The image below demonstrates how CeFi-DeFi arbitrage results in parity between the on-chain price and fair value (i.e. the Binance midprice).

![(Left) Prices on the 3 exchanges Binance, Uniswap and SushiSwap are in equilibria. (Center) A user order significantly moves the price on Uniswap. (Right) A CeFi-DeFi arbitrage brings the prices back to CeFi prices.](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/6c4c5dc1-3507-4baf-b7cc-873177a69dcc/Screenshot_2023-06-05_at_16.44.47/w=1920,quality=90,fit=scale-down)

(Left) Prices on the 3 exchanges Binance, Uniswap and SushiSwap are in equilibria. (Center) A user order significantly moves the price on Uniswap. (Right) A CeFi-DeFi arbitrage brings the prices back to CeFi prices.

CeFi-DeFi arbitrage is possible when on-chain prices move due to a large trade, or when off-chain prices move while on-chain prices stay stagnant (e.g., the off-chain price has moved between blocks).

**Executing CeFi-DeFi Arbitrage**

The simplest form of CeFi-DeFi arbitrage involves two legs of the trade on two separate venues. One leg trades until the on-chain price reaches fair value. If this transaction succeeds then the second leg hedges the accumulated position on another (usually) off-chain venue. CeFi-DeFi arbitrage is not atomic and thus contains several risks and significant barriers to entry:

1. **Risks:**

2. **Inventory risk:** The inventory resulting from the first leg must be warehoused until it’s hedged by the second leg of the trade. A sophisticated hedger might exit the resultant position over time, resulting in the trader holding inventory for a nontrivial amount of time. There is an inherent risk to holding low-liquidity tokens as they are more volatile. CEX liquidity providers may also see that the DEX trade has landed, and move their quotes in anticipation of this flow.
3. **Inclusion risk:** If multiple traders are competing for the same opportunity, there’s a chance that a trader’s on-chain leg doesn’t get included. So the trader’s off-chain hedging strategy needs to consider non-inclusion of the on-chain leg. This problem is further complicated by chain reorgs, which can revert historically confirmed transactions.
4. **Adverse selection:** If a CeFi-DeFi arbitrageur lands a trade on-chain, they outbid all the other arbitrageurs for the trade, signaling that they may have overestimated the opportunity size (i.e. they were adversely selected against/winner’s curse). On the contrary, an atomic arbitrageur is always happy to land their trade since the profit is riskless.

5. **Barriers to entry:**

6. **Inventory management:** It is important for a statistical arbitrageur to have the inventory of a token on both on-chain and off-chain venues. When dealing with low-liquidity tokens, the cost of acquiring and risk of holding the token can outweigh the total opportunity size. Inventory across venues also needs to be rebalanced to prepare for an upcoming trade, and managed following the accumulated position of the revenue leg, imposing additional operational costs.
7. **Latency:** Latency is very important because the trader needs to know the fair value immediately before the block is proposed. This means that the entire path—from the CEX to the trading system to the bundle relay to the block builder to the block relay to the validator—needs to be optimized.
8. **High capital requirements:** A successful CeFi-DeFi arbitrage trader needs a high amount of capital and access to low fees on off-chain venues. On the contrary, to land a successful atomic arbitrage, a trader only needs an efficient smart contract and a good bidding strategy (as the actual trading capital can usually be sourced via flash loans).

As CeFi-DeFi arbitrage is risky and has high barriers to entry, currently 35-77% of the expected extractable value gets sent to the validator by winning searchers.

## Atomic vs. CeFi-DeFi Arbitrage

The critical distinction between atomic and CeFi-DeFi arbitrage is the notion of a **fair value.** This means that, theoretically, CeFi-DeFi arbitrage will have a higher market share due to the following reasons:

1. If the fair value of an asset has changed but the prices on-chain have not changed (e.g., [between two blocks](https://twitter.com/ankitchiplunkar/status/1659217719273877505)), then such an opportunity can only be captured by CeFi-DeFi arbitrage.
2. If the price has moved on-chain (e.g., by a user’s trade), then the EV of CeFi-DeFi arbitrage is higher than that of atomic arbitrage due to lower hedging costs.

![The left image shows the state of the prices after an atomic arb; note there is a difference between the on-chain and off-chain prices. The right image illustrates the state of the prices after a CeFi-DeFi arbitrage; note that all three prices are back in equilibria.](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/9b2b59bb-9e39-4c2e-ac07-7156d676f757/Screenshot_2023-06-05_at_16.46.50/w=1920,quality=90,fit=scale-down)

The left image shows the state of the prices after an atomic arb; note there is a difference between the on-chain and off-chain prices. The right image illustrates the state of the prices after a CeFi-DeFi arbitrage; note that all three prices are back in equilibria.

Let us expand on the second claim by considering two trading venues where the prices have become dislocated. An arbitrage trade looking to capitalize on this discrepancy can be fundamentally decomposed into (1) a revenue leg which trades until the price dislocation is closed (while accounting for some profit margin and trading fees) and (2) a hedging leg that exits the position accumulated in the revenue leg. This framework to characterize legs of an arbitrage can be extended to trades with more than two legs while maintaining the same properties.

Atomic arbitrage involves traders exiting the entire position accumulated in the revenue leg(s) in one single execution, disregarding slippage or other execution costs; this approach leads to significantly negative expected PnL for the hedging leg(s). On the contrary, in CeFi-DeFi arbitrage and broader EV_signal execution, each leg of the arbitrage is evaluated and executed independently with respect to the fair value, allowing the arbitrageur to fully exit the hedging leg over time. However, this strategy introduces the risks and costs discussed above—namely, inventory acquisition and management risks associated with low liquidity tokens. As a result, we empirically observe the cost of this risk in that CeFi-DeFi arbitrageurs bid around 35-77% of the revenue leg to the validator, while atomic arbitrageurs bid 90-99% of revenue.

While holding post-trade risk and maintaining trading inventory introduces complexity to the trade, CeFi-DeFi arbitrage allows one to realize more revenue as they can precisely trade the dislocation to equilibrium and [cheaply hedge the revenue leg](https://twitter.com/0xShitTrader/status/1626071850517254145).

![image](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/02177475-1186-47f2-b10b-8aaf14c0a3e9/Screenshot_2023-05-17_at_10.49.08/w=1920,quality=90,fit=scale-down)

### Empirical Evidence

To demonstrate that the above conclusion holds, we examine some examples of atomic arbitrage and evaluate their execution in the context of CeFi-DeFi arbitrage. To this end, we simulate the expected revenue had the arbitrageur hedged on a centralized venue.

1. [**High Liquidity Token Arbitrage**](https://etherscan.io/tx/0x23456cea5cac07c5ea6bbaa64171a12f303438f6d1efd453a70079703f0e6c9d)

![image](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/2475c02a-16dd-41aa-9d0f-9743a245dad2/Untitled/w=1920,quality=90,fit=scale-down)

At block 16820372, a user submitted a large SNX trade through FlashWallet, dislocating the price from $531.285 to $545.292 on SushiSwap. The fair value for SNX on Binance was $533.488.

1. An atomic arbitrageur traded on this discrepancy earning $21.55 in the revenue leg, while paying $5.76 in hedging costs.
2. If we simulate the same opportunity via CeFi-DeFi, the trader extracts more value—$22.49—through the revenue leg and then hedges with almost zero effective cost. High liquidity tokens such as SNX have nearly zero hedging costs for a volume of 1.4 WETH.

Atomic arbitrage results in $15.79 revenue (before gas), while EV_signal results in $22.49 revenue (before gas) for this trade. With the arbitrageur bidding 91-99% of this trade away to the builder and 35-77% bidding behavior for EV_signal trades, there is a significant profitability cushion for the EV_signal trade.

2. [**Low Liquidity Token Arbitrage**](https://etherscan.io/tx/0x1e88b3465905c95112c69e54bdb1f6364ff1b64460c0c86f374928c2cba5ca0a)

![image](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/ac6c9fa9-9faf-4af9-97b8-caf9aeb77e99/Untitled/w=1920,quality=90,fit=scale-down)

We next analyze a two-leg arbitrage involving a low liquidity token (DSLA token ranked [758](https://coinmarketcap.com/currencies/dsla-protocol/) by market cap).

1. For the atomic arbitrageur, the first leg here is not only the hedging leg but is also responsible for inventory acquisition. This is a common pattern where the arbitrageur is unlikely to hold long-tail assets and thus must acquire inventory to execute the revenue leg—usually at an expensive price. This trade involves a $5.39 hedging leg and is followed by a $10.58 revenue leg. The hedging leg here is expensive, costing over 50% of the revenue, making this trade a prime candidate for EV_signal style execution.
2. If we simulate the same opportunity in the EV_signal framework, the arbitrageur’s revenue increases to $14.66. The arbitrageur must hold DSLA inventory on-chain before executing the trade, however, increasing the trades’ inventory risk; thus, they require a higher profit margin for the trade and bid less than the EV_ordering trade.

Nevertheless, this is still a compelling case for EV_signal execution due to the relatively small notional amount of DSLA traded.

## State of Arbitrage Today

CeFi-DeFi arbitrage can theoretically extract more value than atomic arbitrage. Empirically, we observe that 60% of opportunities (by revenue) are executed via CeFi-DeFi arbitrage. Furthermore, the data demonstrate that atomic arbitrage dominates when either:

1. The primary (liquidity, price discovery) trading venue is on-chain, or
2. The cost of hedging (risk-taking) is significantly higher off-chain.

![Comparison of atomic and CeFI-DeFi arbitrage over Q1 2023. CeFi-DeFi generated $37.8M revenue in Q1 2023 compared to the $25M revenue of atomic strategies. 91-99% of the revenue from atomic arbitrages is paid to the validator for inclusion, whereas only 37-77% of CeFi-DeFi revenue is paid to validators for inclusion. Atomic transactions were sourced from](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/4dfeac05-231d-4277-b309-a4b8f5117a18/Untitled/w=1920,quality=90,fit=scale-down)

Comparison of atomic and CeFI-DeFi arbitrage over Q1 2023. CeFi-DeFi generated $37.8M revenue in Q1 2023 compared to the $25M revenue of atomic strategies. 91-99% of the revenue from atomic arbitrages is paid to the validator for inclusion, whereas only 37-77% of CeFi-DeFi revenue is paid to validators for inclusion. Atomic transactions were sourced from [EigenPhi](https://eigenphi.io/).

While the market size for atomic arbitrage is well-studied and easy to estimate, doing the same for CeFi-DeFi arbitrage is more nuanced. First, a dataset containing all swaps having a `to_addr` corresponding to a known searcher was assembled. Thereafter, swaps identified to be atomic arbitrage (or associated with a sandwich attack) were filtered out using data from EigenPhi. Finally, the revenue of each transaction was determined by calculating the instantaneous markout of the trade with respect to the centralized exchange midprice (the midprice used was derived from the most liquid venue for the given token). We note that our coverage of swaps was not exhaustive (approximately 80% capture), and thus our estimates represent a **conservative lower bound**.

![95% of the atomic arbitrage opportunities are executed on low-liquidity tokens, i.e., the arbitrage includes at least one low-liquidity token. 91% of CeFi-DeFi opportunities are executed on high-liquidity tokens, i.e., all the tokens in the arbitrage are highly liquid.](https://images.spr.so/cdn-cgi/imagedelivery/j42No7y-dcokJuNgXeA0ig/6a0f0466-7847-4311-b45e-e0292a8b2fb3/Untitled/w=1920,quality=90,fit=scale-down)

95% of the atomic arbitrage opportunities are executed on low-liquidity tokens, i.e., the arbitrage includes at least one low-liquidity token. 91% of CeFi-DeFi opportunities are executed on high-liquidity tokens, i.e., all the tokens in the arbitrage are highly liquid.

We see a clear relationship between the liquidity of tokens traded and the type of arbitrage. Specifically, we find that CeFi-DeFi arbitrage is overwhelmingly dominated by trades involving high-liquidity tokens (where we define high-liquidity tokens as being in the top 100 tokens by market cap) and vice versa. This relationship demonstrates that the venue of price discovery for low-liquidity tokens is indeed on-chain, and the cost of hedging off-chain is significantly higher.

For this analysis, we excluded the days during the depeg of USDC as it was an anomalous event. During this period, atomic arbitrage revenue was just shy of $10 million, while CeFi-DeFi revenue was ~$2.8 million. This divergence illustrates how the cost of hedging (risk-taking) was significantly higher during this time, leading to diminished opportunity. That is, CeFi-DeFi arbitrageurs were forced to consider the inventory risk associated with USDC during the depeg, and accordingly scaled back their operations.

## Conclusions and Future Implications

In this post, we characterized and analyzed arbitrage opportunities in the EV_ordering and EV_signal frameworks. We dissected and isolated the concept of risk and illustrated how, oftentimes, executing EV_ordering trades in an EV_signal fashion increases expected PnL. Nonetheless, inventory acquisition and management risks limit EV_signal style execution for low-liquidity tokens. This conclusion is supported empirically, where we observe that low-liquidity tokens are extremely prevalent in atomic arbitrage, while CeFi-DeFi arbitrage dominates high-liquidity tokens. Based on these results, we put forward the following implications for the future of the industry:

### **The Future of Crypto Arbitrage**

As searchers continue to gain off-chain sophistication and off-chain liquidity continues to dominate on-chain liquidity, more arbitrages will be captured via CeFi-DeFi. While temporary short-term shocks (e.g. uncertainty about CEX solvency and liquidity) can temporarily increase the cost of risk-taking and thus favor atomic arbitrage, we believe that the long-term trend will be one of EV_signal > EV_ordering. Furthermore, we note that this analysis does not take into account sandwich attacks. With the rise of [OFAs](https://frontier.tech/the-orderflow-auction-design-space) and ‘_[intent’](https://www.paradigm.xyz/2023/06/intents)_[-based](https://www.paradigm.xyz/2023/06/intents) transactions, the proportion of sandwiches and other frontrunning strategies on-chain will reduce and convert into backrunning arbitrage opportunities increasing the overall revenue share of atomic and CeFi-DeFi strategies.

### The Future of Block-Building and Searching

Block builders will optimize for low latency connections to exchanges and block relays to obtain a more accurate view of off-chain state immediately before block proposal. As EV_signal style trading becomes more competitive, searches will need to develop predictive models of fair value (alpha) beyond the liquid exchange mid-price. We already observed this with a few searchers bidding for blockspace based on predicted fair values.

### The Future of Cross-Chain Arbitrage

This analysis also has parallels in the context of cross-chain trading. Specifically, we believe that concerns around validators simultaneously proposing blocks on multiple chains, such that cross-chain MEV can be extracted atomically, are exaggerated. Atomicity is less important than liquidity in many situations.

In a world where cross-chain transaction bundle atomicity is guaranteed, arbitrage for highly liquid tokens will continue via economically efficient EV_signal style execution. Even for tokens that trade entirely on-chain, well-capitalized actors will be willing to hold capital on multiple chains and execute arbs statistically instead of paying for the guaranteed multi-chain atomic execution.

🙏🏼

Special thanks to [Stephane Gosselin](https://twitter.com/thegostep), Mike Setrin, Lev Livnev and [Alex Nezlobin](https://twitter.com/0x94305) for their helpful comments

## References

1. [New game in town](https://frontier.tech/a-new-game-in-town)
2. [MEV Markets Part 1: Proof of Work](https://mirror.xyz/0xshittrader.eth/WiV8DM3I6abNMVsXf-DqioYb2NglnfjmM-zSsw2ruG8)
3. [MEV Markets Part 2: Proof of Stake](https://mirror.xyz/0xshittrader.eth/c6J_PCK87K3joTWmLEtG6qVN6BFXLBZxQniReYSEjLI)
4. [The Orderflow Auction Design Space](https://frontier.tech/the-orderflow-auction-design-space)
5. [Unity is Strength: A Formalization of Cross-Domain Maximal Extractable Value](https://arxiv.org/abs/2112.01472)