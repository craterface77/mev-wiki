## Introduction

In [Part 2](https://mirror.xyz/0xshittrader.eth/c6J_PCK87K3joTWmLEtG6qVN6BFXLBZxQniReYSEjLI), we discussed the actors in the MEV ecosystem under PBS and why block builders are likely to compete on access to exclusive order flow, whether it’s transactions or MEV bundles.

There are two key stakeholders we’ve left out of the MEV discussion: users and protocols. They too have bargaining power in this ecosystem. In Part 3, we analyze the competitive position of users and how they can capture the value they create.

## Payment for Order Flow

Recall this diagram from Part 2.

![The MEV supply chain under PBS.](https://img.paragraph.com/cdn-cgi/image/format=auto,width=3840,quality=85/https://storage.googleapis.com/papyrus_images/4fbc61750c1554d4b12c4fe6a5283d64e4c4819eba5e29be44ac22b34d19b24c.png)

The MEV supply chain under PBS.

### Capturing MEV Users Create

User transaction flow is one originator of MEV opportunities; user-submitted transactions are often the ones getting sandwiched or backrun. But today, they receive none of the MEV that they create. Users send most transactions directly to the public mempool. Searchers participate in auctions for the right to MEV their transactions, with proceeds going to miners/validators.

This doesn’t have to be the case. Users are the originators of these MEV opportunities, so they can have control over who executes them. The auction for the right to execute a transaction can be run by the user, so the auction proceeds go back to the user rather than going to validators.

A private transaction relay can host auctions for the right to execute transactions. The searchers with access to these auctions must promise to not leak the transaction flow, under threat of losing access to the relay.

In this way, users can capture the proceeds of MEV opportunities they create rather than letting it go to validators. There is nothing validators can do to capture this MEV—transaction originators have full control over where it goes.

![Private MEV auctions can return MEV to its originator.](https://img.paragraph.com/cdn-cgi/image/format=auto,width=3840,quality=85/https://storage.googleapis.com/papyrus_images/b939f1b14b5afb536b0acc08dd3ee029fcfc1fdfbabddd80852da3b37930a92c.png)

Private MEV auctions can return MEV to its originator.

### Private Transaction Relays

Users gain, relative to the status quo, by selling their order flow to a private transaction relay. There are multiple types of guarantees private transaction relays can offer to users to incentivize users to use the relay rather than sending transactions to the public mempool.

- Include transactions for lower than market gas.
    
- Include transactions without frontrunning them. Relays auction to searchers the right to backrun these transactions, collecting the proceeds of the auction.
    
- Include transactions, host private MEV auctions, and return some of the proceeds to users.
    
- Include transactions without MEVving them.
    
- Never include failed transactions.
    

Block builders are incentivized to integrate with and honor the guarantees of these private relays because they represent access to exclusive transaction flow. Any form of exclusive flow is valuable to block builders because it allows them to build more valuable blocks than other block builders.

Private transaction relays can monetize by selling exclusive order flow to builders or by taking a cut of private MEV auctions. Relays can incentivize transaction originators to use them by rebating users.

Wallet companies are the natural aggregators and sellers of order flow. A wallet provider can advertise itself as a "MEV-resistant wallet" or a "gas-rebate wallet" with its default RPC set to its private relay. Wallets can monetize by selling exclusive access to their order flow, rather than by integrating native swaps with toxic fees to the user.

![Payment for Order Flow](https://img.paragraph.com/cdn-cgi/image/format=auto,width=3840,quality=85/https://storage.googleapis.com/papyrus_images/ab1729b9f2d8205ab686ba4ffa7098ebd8c7dcc9aee8266404b4e9656cf534db.png)

Payment for Order Flow

Note how similar this is to payment for order flow in traditional finance. The platform that controls the valuable order flow can decide who to sell it to; in DeFi this is wallets, in TradFi this is retail brokers. End users benefit in the form of execution improvement.

If a dominant wallet like MetaMask sends all of its transaction flow to a single builder, they'll be able to win a large percentage of all blocks. This is very centralizing at the block builder layer.

### Other Transaction Flow Auctions

The mitigation is to auction the flow, so that whoever controls the transaction flow still captures its value but block builders must compete to get access to transactions.

Wallets can aggregate transaction flow and sell it wholesale to block builders, but there are other viable models as well. Transactions can instead be auctioned individually with block builders bidding for the right to execute each one. To keep these auctions permissionless, these transactions can be partially shielded to enforce that auction participants must win the auction to execute transactions.

![Order Flow Auction](https://img.paragraph.com/cdn-cgi/image/format=auto,width=3840,quality=85/https://storage.googleapis.com/papyrus_images/031b937ed77ae851371e0ff647330875b76745bd4f983bbaff175a57a2aae654.png)

Order Flow Auction

This is just as good for the order flow aggregator. The seller of the flow still captures its value, but now block builders compete to get access to transactions. These private transaction flow auctions should pay sellers the marginal value of transaction to builders.

Markets for transaction inclusion move away from all-pay gas auctions to single-price auctions, where excess bids are returned to users. Again, less value flows to validators, decreasing the ETH staking yield.

### Why Now

If controlling order flow is so valuable, why haven't mature markets for it emerged yet? The answer is that PFOF markets make much more sense with PBS under PoS than with PoW. In PBS, block builders are natural buyers of exclusive transaction flow, as their market share depends on their ability to build the most valuable block. The market share of PoW miners is much less affected by that.

So block builders in PBS will play by the rules of order flow auctions to get access to private bundles and transactions. And they can provide execution guarantees to the users.

## Effects on the Mempool, MEV Searching, Block Building, and Validating

### Mempool

PFOF moves Ethereum from a public mempool to private fragmented mempools. Searchers that depend on the mempool (frontrunning / backrunning) may lose access to many opportunities as their target transactions get protected. Other searchers will be unaffected.

### Searching and Block Building

When right-to-execute auctions are privately gated, searchers and block builders with access have a systematic advantage. This raises the barrier to entry for both these businesses. New block builders cannot compete without exclusive transaction flow, exclusive MEV bundle flow, or a world-class MEV extraction capability.

Anyone who sends sufficiently many transactions has an advantage in block building, because their transactions represent exclusive transaction flow. Exchanges and L2 sequencers should be able to save on network fees, either by selling their transaction flow to builders or by building their own blocks.

### Validating

When these markets mature, validators will no longer receive excess value they receive today. This includes:

- User-generated MEV, which gets returned to the user
    
- Gas bids in excess of the minimum for inclusion, as order flow auctions move inclusion auctions from all-pay to single-price.
    

This value is finally returned to its rightful place, as users pay only for the marginal value of the blockspace they consume. This also reduces validator rewards and therefore ETH staking yields.

## Conclusion

While the competitive dynamics will only be discovered after the PBS ecosystem matures, it is clear that users and wallets can capture much more of the value they create by selling access to their transaction flow, similar to PFOF in traditional finance.

Users can capture user-generated MEV by auctioning the exclusive right to execute and MEV them, rather than letting validators run the auction. In general, transaction senders can capture the marginal value of their transactions by auctioning the right to include their transactions to block builders. These markets are specifically enabled by PBS.

Block builders must participate in these auctions to build competitive blocks. Wallets, other infrastructure providers, and entities with lots of on-chain activity are natural sellers of this transaction flow.

Under this regime, value from both user-originated MEV and transactions paying above market gas will get captured upstream of validators. This leads to ETH staking yields that are significantly lower than current projections, which assume that all miner MEV revenue will turn into validator MEV revenue.

In Part 4, we will discuss how MEV-aware protocol design can prevent value leakage from MEV.

Thanks to [@snoopy_mev](https://twitter.com/snoopy_mev), [@0x81B](https://twitter.com/0x81B), [@0xQuintus](https://twitter.com/0xQuintus), [@sxysun1](https://twitter.com/sxysun1), and others for feedback on this part.
