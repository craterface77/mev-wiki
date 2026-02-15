## Introduction

In [Part 1](https://mirror.xyz/0xshittrader.eth/WiV8DM3I6abNMVsXf-DqioYb2NglnfjmM-zSsw2ruG8), we discussed how Flashbots Auction mitigates the negative effects of MEV by introducing a trusted, central relay for searcher bundles. In Part 2, we discuss how MEV markets will evolve when Ethereum moves to PoS consensus.

## Proposer Builder Separation

Ethereum is moving towards in-protocol proposer builder separation (PBS). PBS takes Flashbots Auction to the next level. With Flashbots Auction, searchers bid for specific pieces of priority blockspace (e.g. top of block, immediately in front of / behind target transactions). With PBS, block builders bid for the right to construct the entire block. Validators simply select the block with the highest block bid and propose it.

PBS removes the requirement to be sophisticated as a validator. In the long term, this unlocks protocol designs that require much more complicated and intensive block building, such as danksharding. PBS also makes it harder for staking pool validators to hide MEV revenue from their stakers, since the block builder bids are public. PBS is also viable under PoW, and in fact individual mining pools implement something similar today.

Like how integrating with the Flashbots bundle relay and running `mev-geth` can make a naive miner’s blocks competitive with other miners’, PBS makes naive validators competitive in MEV extraction by unbundling MEV extraction and block building from block proposing. This mitigates the severe centralizing force at the validator layer, where a validator with a slightly better MEV extraction capability and therefore a higher staking reward can gain stake weight very quickly.

In-protocol PBS is far away, so Flashbots is building its own implementation to be ready for The Merge. Under the Flashbots model, validators can run `mev-boost` and get access to blocks built by external block builders via centralized block relays, proposing the one that is greedily optimal for the validator.

While searchers can take advantage of transaction-level MEV opportunities, block builders have access to the full toolset of single-block MEV: transaction ordering, censoring, and opportunistic insertion.

![The MEV supply chain with mev-boost.](https://img.paragraph.com/cdn-cgi/image/format=auto,width=3840,quality=85/https://storage.googleapis.com/papyrus_images/d3d84426150a02be5093319dc5fa46dec680caaf068f2caa3e571f84afaf63d3.png)

The MEV supply chain with mev-boost.

## Centralizing Forces

### Block Builder Competition

Abstracting block building away from validators makes running a competitive validator more accessible. However, the centralizing effects aren’t eliminated; they are just moved to the block builder layer.

Given no MEV opportunities and a public mempool, it’s easy to construct a block competitive with the best possible block (this is the well-studied [knapsack problem](https://en.wikipedia.org/wiki/Knapsack_problem)). Sustainable advantages in the block building business arise when those assumptions are not true.

#### Competing on MEV Bundle Flow

A block builder that can extract MEV more efficiently than others can build more profitable blocks and therefore win more block auctions. There are two main ways a block builder can compete on MEV extraction: have searchers send you bundles, or extract the MEV yourself.

Flashbots' block builder will primarily compete the first way. Flashbots is positioned to be the initial dominant private bundle relay, as they are relatively trusted today and all searchers are already integrated with them. The existing Flashbots bundle relay will exclusively send bundles to the Flashbots block builder, so if searchers continue to exclusively send bundles to `mev-relay`, it will be very difficult for similar block builders to compete with the Flashbots block builder.

Searchers will only send bundles via relays that promise fair auctions and bundle privacy. Other block builders who want searchers to send them bundles will need to be highly trusted, which naturally points to existing infrastructure providers like [Alchemy](https://www.alchemy.com/), [Infura](https://infura.io/), and [bloXroute](https://bloxroute.com/).

To incentivize searchers to onboard, block builders will need to have some market share so searchers need to integrate with them to maximize their chance of inclusion. This can be done by something like systematically overbidding for 1% of blocks to ensure 1% market share. A dominant block builder should protect its exclusivity by incentivizing searchers to not send bundles to other builders. In this system, searchers have more bargaining power than they did before, because they can choose who to send their bundles to.

If a single block builder or colluding set of block builders has enough exclusive bundle flow, it can hide MEV from validators (and therefore ETH stakers) by bidding just enough to win the block auction, with the bid not reflecting the full value of their block. This surplus value can be captured by the block builder, at the expense of validators who no longer receive the full value of the block. The more exclusive bundle flow a block builder has, the more value they can extract from the block auction.

#### Competing on MEV Extraction

Block builders can also extract MEV themselves. Without being limited by the bundle abstraction, block builders can take advantage of MEV opportunities not easily expressible via the existing bundle relay architecture. One clear example of this is end-of-block arbitrage: currently these opportunities land at the top of the next block, even though they are extractable in the current block.

To win blocks, a self-extracting block builder will need to extract more MEV in a given block than the Flashbots block builder. Long-tail searchers in uncompetitive trades may be able to win the relevant blocks. The top searchers for common opportunities, e.g. DEX/CEX mispricings, can build competitive blocks when there aren't many other MEV opportunities in the same block. This type of block builder is natural for top searchers to operate.

These two types of block builders are not mutually exclusive. But independent searchers will be wary of sending bundles to a block builder that also does its own searching.

#### Competing on Transaction Flow

The Flashbots block builder will begin with the advantage of having exclusive access to MEV bundle flow, but as discussed above this advantage may get whittled down over time.

Block builders can also compete by having exclusive access to transaction flow, giving them a larger set of transactions to build blocks from. [Flashbots Protect](https://docs.flashbots.net/flashbots-protect/rpc/quick-start/) is an example of this; by sending a transaction through Flashbots Protect, the Flashbots block builder gets the exclusive right to include the transaction in its blocks, while the user gets no included failed transactions and frontrunning protection.

Markets for transaction flow are about to take off, with significant effects on all actors in the network. We will explore them in Part 3 of this series.

---

The block building business will have high barriers to entry. Participants must have access to private transaction or MEV bundle flow, or a world class MEV extraction capability. It is hard to imagine a world without a small set of dominant block builders, with few opportunities for new players to get in the game.

### Colocation

The MEV searching game looks different in PoS for two main reasons:

- Block proposers are known ahead of time, unlike in PoW. So whether the next block's proposer is using `mev-boost` is known ahead of time.
    
- Block proposals and auctions have fixed times, unlike in PoW where block times are Poisson-distributed.
    

In PoW, searchers are already incentivized to colocate with MEV relays and miners to get the last look at mempool transactions and CEX updates before sending their bundles. With known block proposers and block times, these colocation incentives are stronger because the last look property is more deterministic. Because `mev-boost` has public block builder bids, block builders want to submit their blocks as late as possible to penny the top current bid or hide their bid for as long as possible.

So independent searchers are heavily incentivized to colocate with bundle relays and block builders. And block builders are incentivized to colocate with block relays.

Gains from colocation represent a centralizing force that can manifest in two ways.

- There are natural synergies in vertically integrating the searching and block building businesses.
    
- If vertical integration doesn’t happen, then there are synergies in locating all of these key pieces of infrastructure within the same datacenter.
    

## Competitive Equilibria

In PoW MEV, only searchers and miners are relevant, with miners taking all of the MEV auction proceeds. The distribution of MEV opportunities is different under PoS + PBS because now there are three parties: searchers, block builders, and validators. The introduction of the block builder layer changes the competitive equilibrium.

Recall that independent searchers will send their bundles to dominant block builders because they need to maximize chance of inclusion. That doesn’t mean they have no leverage over these block builders, because block builders only derive value from exclusive bundle flow. A searcher can always threaten to additionally send its bundles to other block builders, reducing the surplus available to a dominant block builder.

A single dominant block builder with exclusive MEV bundle flow is likely to win all of the valuable blocks (blocks with lots of MEV) without having to pay validators the full value of their blocks. Because `mev-boost` is designed with public bids, the dominant builder can bid just slightly more than the next best builder. This reduces the yield ETH stakers receive.

Validators and stakers are harmed the most by this dynamic—they do best in a world where multiple competitive block builders all have thin margins. Validators can incentivize searchers to send their MEV bundles to other block builders. These alternate block builders will not have extractive power but can compete a dominant block builder down to near-zero margins, benefiting validators and stakers. The largest staking pools, e.g. Lido, Coinbase, and Binance, are most likely to incentivize the creation of alternate block builders, as a way to minimize extraction from a single top block builder.

The observed equilibrium will likely depend on the market share of individual searchers and the deals they cut with block builders, and how valuable the MEV bundle flow is relative to other forms of exclusive order flow.

## Trust Assumptions

Every relay in the system (private transaction relays, private bundle relays, block builder relays) and their consumers need to be trusted. Because these relays are centralized, good behavior is enforced by censorship of malicious actors.

## Conclusion

Proposer-builder separation moves centralizing forces from the validator layer to the block builder layer. The main dimensions on which block builders compete are access to exclusive bundle or transaction flow, and in-house MEV extraction capability. Competitive block building requires high levels of sophistication, and it is possible to end up with a single dominant block builder with extractive power.

Independent searchers will have a tougher time under PBS because they will need to figure out where to send their bundles, and some block builders may not operate trusted bundle relays.

The equilibria between searchers, block builders, and validators will be dynamic, because searchers will have some leverage over the other two parties.

In Part 3, we’ll explore markets for transaction flow and how they affect the MEV economy.

Thanks to [@snoopy_mev](https://twitter.com/snoopy_mev), [@0x81B](https://twitter.com/0x81B), [@0xQuintus](https://twitter.com/0xQuintus), [@sxysun1](https://twitter.com/sxysun1), and others for feedback on this part.
