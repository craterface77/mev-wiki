## Introduction

In this series, we consider the design of MEV marketplaces under PoW and PoS consensus. We analyze natural centralizing forces and value extraction mechanisms in these marketplaces. We focus on MEV marketplaces on the Ethereum network, but many of the conclusions are applicable to other blockchains.

In this part, we describe the rise of MEV, early methods for MEV extraction, and Flashbots Auction under PoW.

Readers should be familiar with the MEV ecosystem.

## Early MEV

Early MEV opportunities created by DeFi activity made some blockspace much more valuable than other blockspace. For example:

- A DEX order with large impact on the trading pool creates a backrunning trade opportunity.
    
- A poorly specified DEX order can create opportunity for a sandwich attack, making blockspace immediately before and immediately after the transaction valuable.
    
- Oracle updates enabling liquidations on borrow/lend protocols make blockspace immediately after the oracle update transaction more valuable, since that blockspace can be used to execute the new liquidation opportunity.
    
- Trading activity on DEXs that causes price dislocations between multiple DEXs creates arbitrage opportunities.
    

Without a marketplace for blockspace, searchers attempting to capture MEV opportunities participated in priority gas auctions (PGAs). To capture arbitrage opportunities created in the previous block and available at the top of the next block, searchers would send arbitrage transactions to the mempool, bidding higher and higher gas against each other, up until the point where arbitrage profits no longer exceeded gas costs.

Specific blockspace within a block was particularly hard to target. `Geth` ordered transactions by gas and then randomly, so searchers aiming to land a backrun transaction immediately after a target mempool transaction needed to send their transaction with the same gas as the target transaction, as quickly as possible. Due to the nature of Ethereum’s decentralized gossip network, the optimal strategy was for the searcher to spam the network in the hope of landing immediately after the target transaction. [This update](https://github.com/ethereum/go-ethereum/pull/21358) changed `Geth` to break ties by time received, somewhat reducing the incentive to spam the mempool, but there was still no explicit bidding mechanism for valuable blockspace.

![The MEV supply chain with PGAs.](https://img.paragraph.com/cdn-cgi/image/format=auto,width=3840,quality=85/https://storage.googleapis.com/papyrus_images/b996ad3dc1cbd35302e648c447224cf7446f8187c0d96637e07b8c7702140a33.png)

The MEV supply chain with PGAs.

MEV extraction via PGA has several undesirable externalities to the Ethereum network. These include:

- Failed MEV extraction attempts land in blocks, reducing blockspace available for other transactions and reducing searcher efficiency by increasing their costs.
    
- PGAs in the mempool involve spamming transactions, increasing the load on every node operator.
    
- PGAs cause large fluctuations in market prices for gas, causing some transactions to take a long time to land, and causing systematic overbidding by gas estimators. This has been mostly mitigated by [EIP-1559](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1559.md).
    
- Miners able to extract MEV on their own have a systematic advantage in mining, since they can arbitrarily reorder, censor, and insert transactions anywhere in the blocks they mine. This is a centralizing force for miners because sophisticated MEVving miners have better unit economics.
    

MEV opportunities and their network externalities increase as on-chain activity increases.

## Flashbots

[Flashbots Auction](https://docs.flashbots.net/Flashbots-auction/overview/) created an auction system for blockspace.

At a high level, Flashbots Auction creates auctions for specific blockspace away from the mempool and to a sealed-bid, private auction. It allows expression of preferences more granular than mempool PGA, via transaction bundles.

The canonical example of a MEV bundle is for sandwich attacks: with poorly specified DEX trade B in the mempool, a searcher can specify a bundle consisting of a frontrun transaction A, immediately followed by B, immediately followed by a backrun transaction C, where A and C are only included if both land in this specific way.

In Flashbots Auction, searchers send bundles along with their bids to `mev-relay`, a private Flashbots-operated relay. Participating miners run `mev-geth` to build blocks with access to these bundles. Auction proceeds make it more profitable for miners to run `mev-geth` than vanilla Geth.

![The MEV supply chain with Flashbots Auction.](https://img.paragraph.com/cdn-cgi/image/format=auto,width=3840,quality=85/https://storage.googleapis.com/papyrus_images/9a8bf1e220b9f55383589a92c0f2cc133df339d16ad54f83d28fe886b9ede480.png)

The MEV supply chain with Flashbots Auction.

Relative to mempool PGA, this system is desirable to the network for multiple reasons:

- Failed transactions and bundles don't land in blocks, increasing efficiency of blockspace auctions and reducing wasted blockspace.
    
- Blockspace auctions are moved away from the mempool, reducing unnecessary load on node operators.
    
- The fair value of blockspace is efficiently directed to miners via blockspace auctions, increasing the bounty to secure the Ethereum network.
    
- Current gas prices reflect true demand with less noise.
    
- Targeted MEV is more accessible to searchers. More efficient blockspace auctions mean miners are less incentivized to build out an in-house MEV capability. Instead, miners without an in-house MEV capability can be competitive, reducing centralization at the miner layer.
    

For the most competitive MEV opportunities (e.g. top-of-block atomic arbitrage), searcher bids approach the total size of the opportunity, sending nearly all of the MEV to the miner. Less competitive opportunities have smaller bids.

In theory there can be multiple competing bundle relays, but once there is a single credible well-behaved relay with sufficient market share, no searcher is incentivized to use a new relay unless miners representing a large enough portion of hashpower move exclusively to the new relay. The Flashbots-operated `mev-relay` is dominant, but other systems like [Eden](https://docs.edennetwork.io/) and alternate bundle relays operated by [Ethermine](https://ethermine.org/mev-relay) and [bloXroute](https://bloxroute.com/flashbots-mev-relays/) have some market share as well.

There is also no public auction system for end-of-block MEV opportunities. In theory, no atomic arbitrage opportunities should be available at the top of the next block: it should get extracted by the end of the previous block. Miners should be incentivized to include end-of-block MEV trades so they can capture some of that MEV opportunity, rather than it going to the miner of the next block.

The more MEV that’s extractable via the auction system, the less incentive there is for miners to build out their own MEV capability.

## Trust Assumptions

In its current state, Flashbots Auction introduces various trust assumptions. The bundle primitive and its guarantees introduced by Flashbots Auction rely on good behavior from relay operators and miners.

Malicious relay operators and miners can arbitrarily break up, steal, or censor bundles.

Currently the dominant relay is operated by Flashbots. Miners are incentivized to play by the rules because Flashbots can restrict relay access to miners known to operate maliciously, which reduces the miner’s long-term profitability. In the infinitely repeated game of mining, miners are incentivized to not defect from the Flashbots rules. Flashbots and its employees are incentivized to not defect because of loss of social capital and market share should bad behavior become public.

## Mining: Infinite Game to Finite Game

Recall that miners are only incentivized to play nicely because of the threat of censorship: if a miner is observed breaking or copying bundles, Flashbots can remove their access to `mev-relay`, which reduces long term revenue for miners.

As The Merge approaches and PoW mining moves from an infinite game to a finite game, there are multiple ways miners can monetize their privileged access to `mev-relay`.

- Copy atomically profitable bundles sent by searchers and execute the bundles themselves. This increases miner profitability by taking all the searcher surplus for themselves.
    
- Break up searcher bundles and opportunistically execute those transactions. For example, a sandwich bundle consisting of a searcher frontrun, a mempool DEX order, and a searcher backrun can be executed completely by the miner. The miner can then include the searcher frontrun transaction and backrun it themselves.
    
- Sell access to private bundle data as it comes in to the relay, allowing its customers (paying searchers) to penny their competition in the sealed-bid Flashbots auction.
    
- Sell access to all the private searcher data it’s ever received through the relay, which includes valuable searcher IP: all the bundles and transactions they’ve ever attempted to land, along with their bids.
    

## Conclusion

In this part, we discussed the evolution of blockspace markets under PoW. In Part 2, we will discuss proposed blockspace markets in PoS and how competition in different parts of the MEV supply chain is likely to develop.

Thanks to [snoopy_mev](https://twitter.com/snoopy_mev) and others for feedback on this part.
