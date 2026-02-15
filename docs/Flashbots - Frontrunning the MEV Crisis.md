## Flashbots is a research and development organization formed to mitigate the negative externalities and existential risks posed by miner-extractable value (MEV) to smart-contract blockchains. We propose a permissionless, transparent, and fair ecosystem for MEV extraction to reinforce the Ethereum ideals.


> In this article, we cover the context and motivation for this project. Technical details of our proposed solution can be found on the [ETHResearch forum](https://ethresear.ch/t/flashbots-frontrunning-the-mev-crisis/8251).

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:2000/1*whf9Z6h6piKR43J_PLD0Ug.png)

## What is MEV

[Miner extractable value (MEV)](https://arxiv.org/abs/1904.05234) is a measure devised to study consensus security by modeling the profit a miner (or validator, sequencer, or other privileged protocol actor) can make through their ability to arbitrarily include, exclude, or re-order transactions from the blocks they produce. MEV includes both ‘conventional’ profits from transaction fees and block rewards, and ‘unconventional’ profits from transaction reordering, transaction insertion, and transaction censorship within the block a miner is producing.

The term MEV can be misleading as one would assume it is miners who are extracting this value. In reality, the MEV present on Ethereum today is predominantly captured by DeFi traders through structural arbitrage trading strategies; miners indirectly profit from these traders’ transaction fees. One example of such structural arbitrage opportunities are Uniswap price arbitrage trades: when a Uniswap pool’s assets become mispriced, a profit opportunity is created to arbitrage the Uniswap pool back to parity with other trading venues. Of course, ==rather than letting the trader pay them a transaction fee for the privilege of collecting the arb profit, a miner could simply decide to run this strategy themselves.==

## The MEV Crisis

Transactors on Ethereum express their willingness to pay for inclusion in a block through their transaction’s gas price, and therefore through the transaction fee they indicate they are willing to pay miners. Miners, as economically rational actors, pick the transactions with the highest gas price and order them by gas spend in the block they are producing. The financial system being built on Ethereum creates many ‘pure’ profit opportunities such as liquidations and arbitrages of many kinds. However, these opportunities are finite and episodic, and as such, traders compete to claim them. Right now, this competition is primarily expressed either via frontrunning, or via backrunning:

> Frontrunning (also known as [Priority Gas Auctions](https://arxiv.org/pdf/1904.05234.pdf) (PGAs)): Transaction A is broadcasted with a **higher** gas price than an already pending transaction B so that A gets mined **before** B.
> 
> (eg. to snatch a Uniswap price arbitrage trade to rebalance a pool).
> 
> [Backrunning](https://github.com/ethereum/go-ethereum/issues/21350): Transaction A is broadcasted with a **slightly lower** gas price than already pending transaction B so that A gets mined **right** **after** B in the same block.
> 
> (eg. to execute a DyDx liquidation after a price oracle update that triggers a DyDx loan to go under the required collateralization ratio).

Unfortunately, both frontrunning and backrunning are inefficient and lead to negative externalities such as network congestion (i.e. p2p network load) and chain congestion (i.e. blockspace usage). In addition, this competition for MEV opportunities leads to Ethereum consensus security instability due to the creation of incentives for [time-bandit attacks](https://arxiv.org/pdf/1904.05234.pdf) and permissioned communication infrastructure between traders and miners. Such an infrastructure erodes the neutrality, transparency, decentralization, and permissionlessness of Ethereum today.

While none of these existential risks and negative externalities are new, we find ourselves at a critical junction between alternative futures for Ethereum. A series of events in the past 6 months have lead usage of the network to reach a tipping point:

- **Steadily** [**increasing contract interactions**](https://explore.duneanalytics.com/dashboard/ethereum-usage) (ie. there are more complex transactions on Ethereum than before which increases the absolute amount of MEV up for grabs.)
- **Token market cap** [**exceeding ETH market cap**](https://coinmetrics.io/charts/#assets=btc_log=false_left=CapMrktCurUSD_zoom=1477340857085.4272,1640395584000_formula=W1siRXRoZXJldW0iLCJCbHVlIiwwLCJFVEguQ2FwTXJrdEN1clVTRCJdLFsiVG9wIDUgRVJDMjAiLCJSZWQiLDAsIlVTRFRfRVRILkNhcE1ya3RDdXJVU0QrVVNEQy5DYXBNcmt0Q3VyVVNEK0xJTksuQ2FwTXJrdEN1clVTRCtDUk8uQ2FwTXJrdEN1clVTRCtMRU9fRVRILkNhcE1ya3RDdXJVU0QiXV0) (ie. MEV revenue in ERC-20 tokens is starting to compete with regular transaction fees paid in ETH.)
- **Transaction fees** [**exceeding block rewards**](https://www.theblockcrypto.com/data/on-chain-metrics/ethereum/ethereum-miner-revenue-monthly) (ie. transaction fees have reached unprecedented levels partly due to traders pushing the gas prices up when competing for trading opportunities. It is a clue that MEV-related revenue may surpass block reward for miners.)
- **Adoption of** [**generalized frontrunners**](https://medium.com/@danrobinson/ethereum-is-a-dark-forest-ecc5f0505dff) (ie. an indicator of increased sophistication in MEV extraction.)
- **Adoption of** [**permissioned mempools**](https://samczsun.com/escaping-the-dark-forest/) (ie. another indicator of such sophistication)

These events indicate an accelerating trend towards the foretold existential risks and negative externalities.

## Frontrunning the MEV crisis

> Enter Flashbots

Flashbots is a research and development organization formed to mitigate the negative externalities and existential risks posed by MEV to smart-contract blockchains. We propose a permissionless, transparent, and fair ecosystem for MEV extraction to preserve the ideals of Ethereum.

Our approach to mitigating the MEV crisis can be broken down into three parts: _Illuminate the Dark Forest_, _Democratize Extraction,_ and _Distribute Benefits_. We believe each part is necessary for Flashbots to succeed.

### Illuminate the [Dark Forest](https://medium.com/@danrobinson/ethereum-is-a-dark-forest-ecc5f0505dff)

MEV is currently opaque to users of Ethereum. It requires significant data analysis and in-depth knowledge of smart contracts to understand as it involves transactions with complex, sometimes obfuscated, logic and adversarial games played on several meta-levels (users, traders, generalized frontrunners, miners).

[](https://medium.com/plans?source=promotion_paragraph---post_body_banner_unlock_stories_scribble--40629a613752---------------------------------------)

As more and more security-critical ==infrastructure== moves off-chain, and as chain state and size grows, this problem will only get worse and it will become increasingly difficult to leverage one of the original promises of cryptocurrency: transparency. By Illuminating the Dark Forest, we aim to preserve this original promise. More practically, we aim to allow for the objective assessment of the negative externalities of the MEV crisis and the impact of Flashbots technologies, and for the quantification of user harm caused by MEV extraction in order to provide tooling for builders to reduce their dApp’s surface for MEV extraction.

> Our first step to Illuminate the Dark Forest is quantifying its impact. We’ve built MEV-Inspect for this purpose. It scans Ethereum blocks and enables visualization of MEV metrics over time. We use it to better understand the MEV ecosystem and provide it to the community in an attempt to annihilate information asymmetry.

### Democratize Extraction

MEV Extraction could likely go in a direction where it is centralized to a few players, for instance by being limited to permissioned dark transaction pools that have access to significant hashrate, or through unilateral off-chain deals between large traders and miners.

This power and capital centralization is a key point of security weakness, and erodes core properties of Ethereum: namely permissionlessness and decentralization. We believe that without the adoption of neutral, public, open-source infrastructure for permissionless MEV extraction, MEV risks becoming an insiders’ game. By Democratizing MEV Extraction, we aim to ensure both small and large participants have equal access to low-level financial primitives and that core Ethereum properties are preserved.

> MEV-Geth is our initial effort to Democratize Extraction_._ It is an upgrade to the go-ethereum client to enable a sealed-bid block space auction mechanism for communicating transaction order preference. Fundamentally, MEV-Geth creates a more efficient communication channel for miners and traders bidding for inclusion of their transactions. While the proof of concept of MEV-Geth has incomplete trust guarantees, we believe it is a significant improvement over the status quo. The adoption of MEV-Geth should relieve a lot of the network and chain congestion caused by frontrunning and backrunning bots.

### Distribute Benefits

MEV involves the entire Ethereum ecosystem, from miners, traders, DeFi developers, and, most importantly, Ethereum users. Our preliminary research shows MEV extraction currently disproportionately benefits traders and miners. As MEV extraction continues to grow in scale, we anticipate there will be a need for ~some~ value redistribution towards users and towards system stability

We believe it is essential for Flashbots and the community working alongside us to be thoughtful and deliberate about value redistribution in order to maximize social good. This is particularly true given the aforementioned dangerous economic incentives inherent to MEV which cause existential risks. Not only do we want to mitigate such risks, but also believe it is our responsibility to replace them with virtuous economic cycles that will reinforce Ethereum’s core value proposal by aligning incentives around MEV for all system participants.

## Our Public Commitments

Flashbots arose from the MEV Pi-Rate Ship, a neutral, chain-agnostic, interdisciplinary research collective that supports MEV-related theoretical and empirical research. As an open research organization, we commit today and in the future, to:

- Preserving the core values of Ethereum in what we create, i.e. openness, permisionlessness, decentralization, against the coming MEV crisis.
- Making our research and core Flashbots infrastructure code open source for any community member to contribute to and benefit from.
- Creating sustainable alignment across key actors of the ecosystem by taking into account the needs of users, miners, developers, node operators, public infrastructure operators and developers, contract/dapp devs, and ecosystem researchers.
- Contributing to open-ended ethical research questions in the MEV space, 100% in the public domain.

It is our deeply held belief as both an organization and as the individuals involved that decentralized finance is at a critical crossroads. The substantial amount of value on the table from manipulating user transactions could serve as a centralizing force, damaging to consensus stability, and harmful to users of any system where such manipulation is valuable. MEV could grow to benefit a few at the expense of many, at the expense of the value of cryptocurrency itself.

Or, this value could stand to benefit all users, enhancing the security of a new generation of financial infrastructure that avoids the mistakes of its structurally unfair predecessors. By bringing MEV extraction and tooling into the open, by funding public research to answer open questions around MEV, and by using our organizational capital to align the incentives of all ecosystem participants, it is this new generation of fair infrastructure we aim to lay the foundations for.

This is a call to action. We can’t wait to have you join us.

## Learn more

> **Review our technical proposal on ETHResearch**
> 
> We discuss the details of the Flashbots organization, our research roadmap, and the technical details of the initial projects, MEV-Inspect and MEV-Geth on the [ETHResearch forum](https://ethresear.ch/t/flashbots-frontrunning-the-mev-crisis/8251).
> 
> **Subscribe to the MEV Ship Calendar**
> 
> You can follow the latest Flashbots updates and events by subscribing to our [MEV Ship Calendar](https://calendar.google.com/calendar/embed?src=c_7lurqn12ecl64li0ms3kse5vok%40group.calendar.google.com): join us on our semi-monthly community call “MEV Ship Treasure Map Roast”, semi-weekly core dev call, for our weekly research workshop, and for the upcoming unconference: MEV.wtf
> 
> **Engage with us**
> 
> Join the Flashbots community [on Discord](https://discord.gg/7hvTycdNcK) or contact us at info@flashbots.net

Flashbots is stewarded by [Scott Bigelow](https://twitter.com/epheph), [Phil Daian](https://twitter.com/phildaian), [Stephane Gosselin](https://twitter.com/thegostep), [Alex Obadia](https://twitter.com/ObadiaAlex), and [Tina Zhen](https://twitter.com/tzhen). We exist thanks to the continued support of members of the MEV Pi-Rate Ship and [Paradigm](https://twitter.com/paradigm).

Special thanks to [Andrei Anisimov](https://twitter.com/anisimov_andrei), [Ivan Bogatyy](https://twitter.com/IvanBogatyy), [Vaibhav Chellani](https://twitter.com/vaibhavchellani), [Brock Elmore](https://twitter.com/brockjelmore), [Georgios Konstantopoulos](https://twitter.com/gakonst), [Jason Paryani](https://twitter.com/jparyani), [Alejo Salles](https://twitter.com/fiiiu_), [samczsun](https://twitter.com/samczsun), [Austin Williams](https://twitter.com/onewayfunction) for their contributions on MEV-Geth and MEV-Inspect, and [Sunny Aggarwal](https://twitter.com/sunnya97), [Surya Bakshi](https://twitter.com/suryabakshi), [Phillippe Castonguay](https://twitter.com/PhABCD), [Tarun Chitra](https://twitter.com/tarunchitra), [Dan Elitzer](https://twitter.com/delitzer), Lev Livnev, [Charlie Noyes](https://twitter.com/_charlienoyes), [Dev Ojha](https://twitter.com/valardragon), [Dan Robinson](https://twitter.com/danrobinson), [Mark Tyneway](https://twitter.com/tyneslol), and [Micah Zoltu](https://twitter.com/MicahZoltu) for their feedback on MEV-research.

