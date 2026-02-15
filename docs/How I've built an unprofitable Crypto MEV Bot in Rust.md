MEV bots are money-printing machines. At least in theory. For the last ~year, I’ve been working on the MEV crypto bot for Ethereum EVM-compatible chains. In this blog post, I’ll describe the ins and outs of trying to get into the MEV game with a limited web3 skillset and relatively small capital.

_Disclaimer: The information provided in this blog post is for educational purposes only and should not be treated as financial advice. MEV is extremely risky, and you’re likely to lose all the funds that you allocate._

## MEV bots 101

In this post I hope to provide a hands-on perspective on what it means to try playing the MEV game in 2024.

After publishing [my recent posts](https://pawelurbanek.com/revm-alloy-anvil-arbitrage) [about MEV extraction](https://pawelurbanek.com/mev-yul-huff-gas), I’ve received several emails asking for more details. This post should provide more in-depth answers to your questions. We will discuss the chances to profit, cover the required technical skillset, and infrastructure costs.

For those unfamiliar with MEV (Maximum Extractable Value), in brief, it’s a value (usually denominated in ERC20 tokens) that can be obtained from the imbalances and inefficiencies in blockchains by highly specialized actors.

A basic example of MEV strategy is taking advantage of price discrepancies between on-chain DEXes (decentralized exchanges). E.g., buying USDC for WETH on a UniswapV2 pool, and selling it on a Sushiswap pool for more WETH. It’s called an _“atomic arbitrage”_ because a custom Smart Contract can execute this trade in a single atomic transaction.

Another strategy called _“sandwiching”_ consists of _“wrapping”_ another user’s swap (extracted from the mempool) into bot-generated transactions. The first tx skews the buy price for user, and 2nd sells the asset with a profit. It’s worth mentioning that sandwiching is detrimental to Defi users who don’t use MEV protection.

There are also so-called long-tail MEV strategies, which are usually less competitive. But, more on that later.

## How to get started with MEV?

YouTube and Google SERP are ripe with scam videos and articles on _“How to build a passive income MEV bot (with ChatGPT)”_. These are always low-effort schemes designed to steal your crypto assets. A _“MEV bot smart contract”_ deployed via [Remix](https://remix.ethereum.org/), will just send all the ETH that you want _invest_ to the attacker. At least Remix is now displaying a helpful alert when copy/pasting the Solidity codebase. But apparently people are still falling for these types of scams.

![Remix editor scam alert](https://pawelurbanek.com/assets/remix-alert-mev-f7e0908126b8e62c6964a3fe31d52054f33eee7cb66bdd051be6ac4b8f940939.png "Remix editor scam alert")

So, how can you get started with MEV and not lose your assets with a few clicks?

An MEV bot consists of the client-side code (can be NodeJS, Python, Go, Rust), a set of custom smart contracts (Solidity/Vyper/Yul/Huff), and, optionally, a fleet of proprietary blockchain nodes.

Let’s discuss these layers, starting with the client-side.

### Client-side

Choosing the client-side framework is arguably the most impactful technical choice you have to make. MEV bots are vastly more complex on the client side compared to the on-chain Solidity code. My bot currently consists of 10k+ lines of Rust code and only ~500 LOC for Solidity Smart Contracts.

I’ve started learning MEV with the popular [simple-arbitrage by Flashbots](https://github.com/flashbots/simple-arbitrage) written in NodeJS. When the bot’s complexity grew, I found myself spending a lot of time debugging runtime errors, and it got me into learning Rust.

I’ve since gotten kind of hyped for Rust and started using it also in my non-web3-related projects. Another advantage of Rust is its superior performance and advanced tools like [Reth](https://github.com/paradigmxyz/reth) or [Revm](https://github.com/bluealloy/revm).

But if you want to get started, the best approach is to execute some of the open-source bots written in the tech stack you already know. I’m sure there are MEV teams finding success regardless of the client-side tech.

MEV bot client process usually lives on a VPS scanning blockchain for profit opportunities. But how do we connect to the blockchain?

### MEV node infrastructure

If you’ve ever interacted with Defi, you’ve probably started from a popular [Metamask wallet](https://metamask.io/). It hides the difficulty of choosing your blockchain RPC by defaulting to the publicly available Infura nodes. You can check out [Chainlist](https://chainlist.org/) for public node endpoints of different blockchains.

However, you’ll probably not be able to run your bot on them. These public endpoints have strict rate-limiting applied, so even basic scouting for data is likely to put you on a blacklist. I’ve had success with ~$50/month plans from [ChainStack](https://chainstack.com/) and [Alchemy](https://www.alchemy.com/). They support a variety of EVM blockchains, so you can test your strategies in different environments.

But, once you find a chain where your strategies yield a consistent profit, you’ll probably want to keep your bot running 24/7. It’s rare to find an MEV opportunity lying around. >99% of them disappear in the next block (with the exception of not yet discovered long-tail MEV, but more on that later). It means that a competitive MEV bot has to continuously scan the blockchain for fresh opportunities. This mode of operation will likely exhaust even $1000+ 3rd party RPC plans within a few days.

To _get serious_ about MEV, you should run your blockchain node without request limits. So, let’s quickly discuss the required skillset and costs.

#### Mainnet full node

The current Ethereum Mainnet node running costs oscillate around ~$150/month. I think it’s the best way to get started testing your bot’s data-gathering strategies and continuous mode of operation. You can [read my blog post](https://pawelurbanek.com/ethereum-node-aws) for info on how to run your proprietary full node. Mainnet Geth client is well established, so there are few surprises. You should be able to get the Mainnet synced in ~24h.

The downside of operating on the Mainnet is that it’s currently the most competitive of the networks. I’ve had a brief adventure with MEV extraction in the pre-EIP-1599 era. Now, long gone are the times when you could profit >1$ on a simple UniV2 atomic arbitrage. Nowadays, mainnet bots seem to be competing for a fraction of a cent just to win the Flashbots PGA (Price Gas Auction) for popular opportunities.

#### Sidechain nodes

There’s a better chance of finding competitive opportunities on layer 2s or standalone EVM networks. This means that you’ll have to synchronize other types of nodes. Layer 2 nodes (like Base, Arbitrum, Optimism, and ZKSync) require a Mainnet full node to synchronize. Standalone networks (BSC, Polygon, Fantom) need just their full node to operate.

One issue with synchronizing non-mainnet nodes is that they are sometimes sparingly documented and less stable. So prepare for more surprises.

Finally, my bot operated on the Mainnet and two sidechains that I managed to squeeze on a single 192GB RAM VPS. It took some trial and error, but monthly costs to run this infra closed in ~$750 USD.

### Mev bot Smart Contracts

The last part of the MEV bot stack is Smart Contracts deployed on-chain. Some of them are used for transaction execution, and some to facilitate data gathering. A usual pattern is to deploy an _executor_ contract that holds your capital. As I’ve mentioned before, you’ll reading _a lot_ more Solidity than writing it. To construct MEV strategies, you have to understand the inner workings of Defi protocols and use client-side code to build and submit transactions. But the executor smart contract is orders of magnitude simpler than the client-side code.

If you’re just starting with MEV, I’d highly recommend going lower level and learning at least the basics of [Solidity Yul](https://docs.soliditylang.org/en/latest/yul.html) and [Huff language](https://huff.sh/). Their primary purpose is to develop strategies that are more gas-efficient; check out [this post](https://pawelurbanek.com/mev-yul-huff-gas) for more info. _Gas golfing_ is more important on the Mainnet because of higher transaction costs. But building at least the simplest project in [Yul](https://github.com/pawurb/yul721) and [Huff](https://github.com/pawurb/huff721) will vastly increase your knowledge of EVM’s inner workings. And it might translate into discovering some competitive edge.

## How to compete with MEV bots?

Now that we’ve covered MEV bot stack layers let’s discuss what running it in practice is like. I categorize EVM networks into 3 types, each with different betting strategies and risk profiles.

### Mainnet - Flashbots

Fighting for MEV on the Ethereum Mainnet is unique because of the block space auctioning system pioneered by [Flashbots](https://www.flashbots.net/). You can spam MEV transactions to compatible block builders, but they will be published only if you score the profit. So, in theory, operating your bot on the Mainnet does not pose any risk. In practice, many MEV bots have been rekt by even more cleverer bots (e.g., with [Salmonella attack](https://github.com/Defi-Cartel/salmonella)). Atomic transactions are much less risky.

The presence of Flashbots stack on the Mainnet means that you can test different strategies relatively safely with reduced risk of loss. Another advantage is that you won’t get frontrun. I think the best way to get started with MEV is to submit a few Mainnet transactions with negative profit, to learn the necessary tooling. Later, all that’s left is to increase your profit margins.

### Mempool side chains

[BSC](https://www.bnbchain.org/en), [Polygon](https://polygon.technology/), and [Fantom](https://fantom.foundation/) are examples of networks that have a similar operating model to the Mainnet but without the Flashbots stack. It means that there’s no mechanism to prevent your failed transactions from getting executed. Depending on the network congestion, each failed txs will cost a few cents (if you’re not PGAing i.e. betting higher gas prices).

These networks also have a shorter block time of ~2 seconds instead of ~12s on the Mainnet. It means that performance optimizations are more important. Winning the MEV profit is a combination of quickly constructing the transaction, and paying high enough gas price. Trying to compete on these networks led me to develop multiple optimizations in my Rust client-side code. You can read about them [in this blog post](https://pawelurbanek.com/revm-alloy-anvil-arbitrage).

An issue with mempool networks without the Flashbots infrastructure is that you’ll likely get frontrun. Imagine finding a stale $100+ opportunity and meticulously crafting a payload to score it. But a mempool scanner bot just steals it by copying and frontrunning your transaction…

![Hercules was frontrun](https://pawelurbanek.com/assets/hercules-frontrun-11f91ef3f5ed449de324730d69bc41888ee1e08968fbe9e64729f45c69a65d9f.jpg "Hercules was frontrun")

A proxy contract does not help. Scanners are smart enough to extract and replicate internal transactions.

### Centralized sequencer chains

[Base](https://www.base.org/), [Optimism](https://www.optimism.io/) and [Arbitrum](https://arbitrum.io/) use a different operating model where mempool is not publicly available. Instead, transactions are submitted directly to the centralized _sequencer_. In addition to client-side performance, colocating with a sequencer is necessary to score N+1 block placement.

Centralized sequencer networks make a range of MEV techniques, e.g., sandwiching, impossible. There’s no mempool from which to extract the _meat_ transaction. An upside is that you can now confidently submit payloads without the risk of getting frontrun by a mempool scanner bots.

Winning on these chains requires landing your transaction in the N+1 block and paying a high enough gas price to outbid the competition. And just like that, your MEV bot is now printing money… or is it?

## How to PGA?

Finding opportunity, calculating optimal profit, constructing tx payload, and submitting it quick enough to land an N+1 block. Once you sort out these engineering quirks the MEV game comes down to the gas betting.

Transactions in a single block are sorted by their gas price. This means that if two MEV bots target the same $100 opportunity, the one that pays $1 in gas will lose to the one that pays $10. But on the EVM networks without Flashbots, you’ll also pay a much higher cost for failed tx if you are betting on gas prices.

For example, on the Base Chain, a failed transaction will cost you a fraction of a cent. But if you’re ready to pay $50 to score $100 arbitrage, the price for a reverted transaction could be ~$10. It means that participating in a PGA auction could quickly drain your capital.

I’ve had numerous approaches trying to find a profitable strategy in the betting game. It always followed the same pattern: few successful txs, with subsequent dozens of failures draining my capital. I think that established MEV bots are highly territorial predators. So, once they detect competition they reduce their profit margins to get rid of it.

I’ve regularly seen opportunities of $1000 profit, where the winner bets over $900 in gas (i.e. scores $100 profit), but the runner-up takes the loss of over $50. I’ve operated on capital of ~$1000, so I could not afford to keep up with this game. There’s an interview with MEV searcher explaining how he likes to _“squeeze out_” any emerging competition. And can afford to go a few weeks or even months without profit to do it. Maybe an optimal amount to bid on MEV opportunities is 101% of profit. Until you slaughter the competition.

So, if you’re a solo searcher with limited capital, your best bet could be to target a long-tail MEV where there’s no PGA.

## How to find long-tail MEV opportunities?

In hindsight, my biggest mistake was focusing on popular strategies. My Ruby/Web performance consulting background led me into the rabbit hole of micro-optimizing performance aspects of my bot. But any profitable tweak I’ve found quickly disappeared when the competition caught up.

The real edge could be not optimizing your bot against the competition but instead developing a framework for finding fresh MEV opportunities. Going forward I’m planning to focus more on new protocols and configuration changes likely to produce MEV. Check out [this post by Bertcmiller](https://www.bertcmiller.com/anatomy_of_mev.html) on how this process works in practice. This [talk by 0xDmitry](https://www.youtube.com/watch?v=lOku3SguPY0) also offers interesting insights on this approach.

_[Update] I’ve [published an article](https://pawelurbanek.com/long-tail-mev-revm) showcasing [mevlog-rs](https://github.com/pawurb/mevlog-rs) a Rust CLI tool for querying blockchain and discovering long-tail MEV opportunities._

Any profitable MEV is likely to eventually get eaten up by PGA. So, being the first to score fresh opportunities could be one way to scale your profits.

### Long-tail MEV examples

I did manage to find opportunities in some _weird edge cases_, but it always evaporated after a few days of yielding decent profits. If you’re planning to tweak a publicly available bot, here’s a list of ideas:

- ERC20 tokens with a custom transfer mechanism
- tokens with a custom buy-back feature
- triangular arbitrage with unpopular token pairs
- pools with small liquidity
- freshly deployed tokens/pools
- freshly deployed DEXes
- forks of popular DEXes with different swap logic

## Miscellaneous

If you’re still not discouraged and want to start your _MEV journey_, here’s a few random tips.

### Automate repetitive tasks and release flow

Unlike any other IT field I’ve worked in, MEV offers an almost instant feedback loop. While working on clients’ projects or [my SAAS](https://abot.app/), I measured feedback in weeks until a deployed change started affecting the profits. An MEV bot can yield a massive profit literally seconds after a release.

It means you iterate extremely quickly, deploy and test dozens of changes each workday.

I recommend spending more than usual time on optimizing the release flow. It’s likely to pay off because of this insanely quick feedback loop. My release pipeline is an ungodly hybrid of shell scripts with rsync, screen, systemd, and cron. But it gets the job done. I can tweak several feature flags and quickly release a new version to different EVM chains.

The same goes for setting up and managing blockchain node processes. You’re likely to start the sync process multiple times from scratch. So, preparing some reusable Ansible playbooks is probably worth it.

### Killswitch

Professional MEV teams have members spread across the globe to cover all the time zones. I have a killswitch.

Waking up with less capital than you’ve had when going to sleep is no fun. There are endless edge cases on why your MEV bot could start suddenly losing money. A simple protection is to check the balance of your contracts on each block and deactivate the bot if it goes below a threshold. Please implement this feature and thank me later.

### Gather data

A technique that allowed me to increase the success rate was an analysis of historical transactions. I’ve configured an SQLite database to gather the following data:

- profit
- volume
- tokens and pools
- fees
- block metadata
- time needed to dispatch tx payload
- gas prices
- and more…

Running your bot even in a mode where it loses capital might lead to discovering profitable patterns. Gather all the data points you can think of, even if they might seem pointless.

### Find friends

I did not manage to find a fren to cooperate. Working alone makes it difficult to keep a fresh perspective. Also, burnout is more likely.

What helped me were semi-technical mastermind groups with people who are crypto natives. Confronting your ideas with someone who understands the environment is always helpful. But working with a full-time partner will likely increase your chance to succeed.

### Learning materials

Below, I’m sharing learning materials and repos I’ve found helpful when starting. They should be _safeish_ to interact with, but please do your own due diligence:

- [Cyfrin Updraft](https://www.cyfrin.io/updraft)
- [DegenCode](https://www.degencode.com/)
- [Solid Quant](https://medium.com/@solidquant)
- [simple-arbitrage](https://github.com/flashbots/simple-arbitrage)
- [simple-blind-arbitrage](https://github.com/flashbots/simple-blind-arbitrage)
- [subway](https://github.com/libevm/subway)
- [rusty-sando](https://github.com/mouseless-eth/rusty-sando)

## Summary

I hope this perspective of someone giving MEV a serious try but not really _wagmi_ yet was interesting. These last months of fighting for every cent of profit were fun.
