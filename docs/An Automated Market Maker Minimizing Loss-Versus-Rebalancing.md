# An Automated Market Maker Minimizing Loss-Versus-Rebalancing

[Conor McMenamin](https://arxiv.org/search/cs?searchtype=author&query=McMenamin,+C), [Vanesa Daza](https://arxiv.org/search/cs?searchtype=author&query=Daza,+V), [Bruno Mazorra](https://arxiv.org/search/cs?searchtype=author&query=Mazorra,+B)

> The always-available liquidity of automated market makers (AMMs) has been one of the most important catalysts in early cryptocurrency adoption. However, it has become increasingly evident that AMMs in their current form are not viable investment options for passive liquidity providers. This is large part due to the cost incurred by AMMs providing stale prices to arbitrageurs against external market prices, formalized as loss-versus-rebalancing (LVR) [Milionis et al., 2022].  
> In this paper, we present Diamond, an automated market making protocol that aligns the incentives of liquidity providers and block producers in the protocol-level retention of LVR. In Diamond, block producers effectively auction the right to capture any arbitrage that exists between the external market price of a Diamond pool, and the price of the pool itself. The proceeds of these auctions are shared by the Diamond pool and block producer in a way that is proven to remain incentive compatible for the block producer. Given the participation of competing arbitrageurs to capture LVR, LVR is minimized in Diamond.  
> We formally prove this result, and detail an implementation of Diamond. We also provide comparative simulations of Diamond to relevant benchmarks, further evidencing the LVR-protection capabilities of Diamond.  
> With this new protection, passive liquidity provision on blockchains can become rationally viable, beckoning a new age for decentralized finance.

