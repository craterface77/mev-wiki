Launching the attack: the green letters look just like on TV

**This post is a deep-dive into programmatically trading on the Ethereum / Bancor exchange and exploiting a game-theoretic security flaw in** [**Bancor**](https://www.bancor.network/?ref=hackernoon.com)**, a high-profile smart contract on the Ethereum blockchain. The full code can be found at** [**https://github.com/bogatyy/bancor**](https://github.com/bogatyy/bancor?ref=hackernoon.com)**. We collaborated with the Bancor team to make sure the current exploit is protected against, although for a little while there would still be a chance to make some beer money for educational purposes.**

Imagine trying to hack Bank of America — except you can read all of their code in advance, all of their transactions are public, and if you steal the money it’s irreversible. Sounds like a paranoid worst-case scenario? Well, this is exactly the setup Ethereum smart contract developers have to deal with every day. Bitcoin and the blockchain technology unlocked tremendous possibilities in international payments, and the Ethereum further magnified it by allowing to manage these payments through programs called **_smart contracts_**. However, smart contracts also give hackers a much easier setup for attacks.

Front-running is one such attack. The term originated in the stock market, back in the days when trades were executed on paper, carried by hand between the trading desks. A broker would receive an order from a client to buy a certain stock, but then place a buy order for themselves in front. That way the broker benefits from the price increase at the expense of their client. Naturally, the practice is unfair and was outlawed.

On the blockchain, the problem becomes a lot more severe. First, all the transactions are broadcast publicly. More importantly, blockchain participants across the world are not bound by the same relationship as a broker and their client, so attackers can exploit their knowledge of a pending transaction with impunity.

[](https://hackernoon.imgix.net/hn-images/1*TxVByaqq8P2LoK7VE-3s_A.png "Download image")

![](data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20version=%271.1%27%20width=%271200%27%20height=%27800%27/%3e)![image](https://hackernoon.imgix.net/hn-images/1*TxVByaqq8P2LoK7VE-3s_A.png?w=3840)

If you squint hard enough, you can imagine these guys trying to front-run each other

Several months ago, researchers at Cornell [uncovered](http://hackingdistributed.com/2017/06/19/bancor-is-flawed/?ref=hackernoon.com) that Bancor, an ICO that spectacularly raised over $150M in funding over a few minutes, was vulnerable to front-running. They pointed out that miners would be able to front-run any transactions on Bancor, since miners are free to re-order transactions within a block they’ve mined. While the Bancor team gave a [thoughtful response](https://blog.bancor.network/this-analysis-of-bancor-is-flawed-18ab8a000d43?ref=hackernoon.com), up until very recently, there has not been any progress on fixing the issue (more on that later).

Our research goes a step further. In fact, we show that it is both possible and practical to front-run Bancor **as a non-miner**. Which means you don’t need to the lucky miner who happens to mine the block with a Bancor trade to profit from front-running. You simply need to be a regular user monitoring the blockchain to perform this attack.

Surprisingly, the vulnerability does not seem to have been exploited so far (front-running is readily identifiable on the blockchain), so in this post we’ll examine exactly how one implements such an attack. Turns out, all it takes is about 150 lines of Python to get a working front-running algorithm. We also ran simulations to determine how much money one could make from front-running consistently (spoiler: **an attacker could have had a ~117% ROI on the money they invested into the attack over July and August, chipping away from other Bancor users**). Finally, I executed the attack against a single trade, making**~$150 net of all fees**, after which I returned the money to the person I front-ran and stopped the program.

Now, I know that relinquishing a working trading strategy would be a cardinal sin to any trader, but as it turns out, I am more curious than greedy. Implementing and countering attacks is not only a fascinating game, but also the cornerstone of advancing cryptographic security. Most importantly, I believe in the long-term impact of the blockchain ecosystem, and for the blockchain economy to fully develop, vulnerabilities like this need to be understood and protected against.

So let’s dig in.

### Background

Bancor is a protocol for trading and pricing Ethereum [ERC-20 tokens](https://ethereum.org/token?ref=hackernoon.com), as well as the eponymous token, abbreviated as BNT, with the current market capitalization of approximately 180 million dollars. The core problem Bancor solves is as follows: normally, for a trade to happen, there has to be a buyer and a seller, having opposing desires (to buy and to sell) at the same moment in time. This limitation may be fine for large publicly traded stocks, but for long-tail crypto tokens this can create a serious inconvenience.

Bancor solves this problem by allowing anyone to trade against a public smart contract, which offers an automatically calculated token price following a precise formula. Essentially, Bancor is fulfilling the role of market-makers in traditional finance. The smart contract has an Ethereum reserve, and as more people buy the token, reserves grow and the price goes up. Consequently, when people sell, the contract adjusts the price to go down, so that the reserve is never depleted entirely. Unlike most other exchanges, where trades are managed off-chain, with Bancor every order is a self-contained Ethereum transaction (money + data).

Unfortunately, the current setup contains a flaw, allowing anyone to front-run large transactions and make guaranteed profit. Let us expand on what makes the attack possible. When somebody **_broadcasts_** a transaction on the Ethereum network, it becomes available to other nodes almost immediately as a **_pending transaction_** and is added to the common queue, but it is not **_confirmed_** until the **_block confirmation hash_** is found by some miner (thus confirming all the transactions in that block), which tends to happen once every ~20 seconds. Further, up until the block is confirmed, the order of the pending transactions is up for grabs, and miners basically [sort transactions](https://github.com/ethereum/go-ethereum/blob/290e851f57f5d27a1d5f0f7ad784c836e017c337/miner/worker.go?ref=hackernoon.com#L492) by how much they’re paid per **_gas_** (that is, per unit of computation they’ll have to perform).

This discrepancy creates an attack vector: any user running a full-node Ethereum client can spot a pending transaction and insert their own transaction in front of it by paying more per gas. If you see a large BUY is about to happen, you know the BNT price will increase (following their deterministic formula), so if you buy in before that transaction you get an instant appreciation of your tokens and a guaranteed return on your investment. Similarly, if somebody sent out a pending SELL, an attacker can sell their tokens in front.

[](https://hackernoon.imgix.net/hn-images/1*J6_Zn2Pizv9b-G-jKQMekQ.jpeg "Download image")

![](data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20version=%271.1%27%20width=%271200%27%20height=%27800%27/%3e)![image](https://hackernoon.imgix.net/hn-images/1*J6_Zn2Pizv9b-G-jKQMekQ.jpeg?w=3840)

Ethereum blockchain (ordered) and pending transactions (partially ordered, but possible to get in front)

Given we will be implementing our attack using Ethereum client API, now would be a good time to take a step back and give a general overview of how the Ethereum distributed applications (or DApps for short) landscape looks like. At a high level, implementing DApps is fairly similar to regular Web applications. The backend is a smart contract, running on the Ethereum blockchain, typically implemented in Solidity and then deployed to the network. Then there is client software, which interacts with the backend by sending transactions.

Just like in the regular world, smart contracts (the backends) receive most of the attention, with many high-profile smart contracts and several high-quality developer guides appearing recently (personally, what I’ve found most useful was fiddling with the examples from [Solidity official docs](https://solidity.readthedocs.io/en/develop/solidity-by-example.html?ref=hackernoon.com), as well as great intro guides by [Hudson](http://hudsonjameson.com/training/ic3-2017?ref=hackernoon.com) and [Karl](https://karl.tech/learning-solidity-part-1-deploy-a-contract/?ref=hackernoon.com)). However, on the front-end side, I currently do not know of any non-trivial client-side applications (an example of such an application would be a [decentralized poker client](https://arxiv.org/abs/1701.06726?ref=hackernoon.com), where the majority of compute has to happen off-chain, on the players’ machines). Right now, the only way for users to interact with smart contracts is to send transactions manually, either by running their own full node (for example, by using the `geth` client), or by relying on third-party web services (like MyEtherWallet). Clearly, this would have to change: the current situation is about as convenient as manually sending POST requests through Telnet every time you wanted to browse the Web.

### Easy Mode: high-frequency trading by hand

The simplest way to confirm the vulnerability is by hand. We will not need any tools except a Web browser and a wallet with some Ether.

First, separate your Ether between two wallets equally (it will not work from a single wallet). Second, go to MyEtherWallet and, following [Bancor purchase instructions](https://blog.bancor.network/experimental-simple-bnt-purchasing-aafd51d03b30?ref=hackernoon.com), set up two BUY transactions (do not click Send yet!). You should prepare two equivalent transactions from both wallets, but make sure gas price on the first wallet is lower than on the second.

[](https://hackernoon.imgix.net/hn-images/0*1CJ7DJC7lhboVizO. "Download image")

![](data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20version=%271.1%27%20width=%271200%27%20height=%27800%27/%3e)![image](https://hackernoon.imgix.net/hn-images/0*1CJ7DJC7lhboVizO.?w=3840)

Sending ETH to the Bancor purchase contract automatically returns BNT to your account. Note: mind the gas!

Now, when it’s set up, click “Send” on the first wallet (with the lower gas price), and then “Send” on the second wallet, with the higher gas price. If you did everything right, the transaction that was submitted second would actually be processed first, and get more `BNT` per same deposit!

[Transaction 1](https://etherscan.io/tx/0xe09de9c9093565e645eb93a74d86c390cb96bec88131abcbefc1b0b889ff5241?ref=hackernoon.com) _(transaction from wallet 1, submitted first, fulfilled second)_

`BNT tokens received: **11.014424**733254973428`

[Transaction 2](https://etherscan.io/tx/0x4bf702fef14c4da5beae7f676d93f80316cb872390c480450f43e968dba4c53a?ref=hackernoon.com) _(transaction from wallet 2, fulfilled first, should get better price)_

`BNT tokens received: **11.014423**186864343663`

Notice the letdown: while the front-running transaction should’ve gotten a better price, it actually got a slightly worse one. Upon careful investigation, I realized this is because the [rather arcane implementation](https://github.com/bancorprotocol/contracts/blob/501564a18c1ddfdfaf5538c98ec383264b5efd6e/solidity/contracts/BancorFormula.sol?ref=hackernoon.com) of the Bancor formula can deviate quite significantly from the [theoretical formula](https://drive.google.com/file/u/1/d/0B3HPNP-GDn7aRkVaV3dkVl9NS2M/view?ref=hackernoon.com), especially for smaller amounts (specifically, error in 4th digit for `0.1 ETH` or roughly $20 transactions). After querying the `[BancorFormula](https://etherscan.io/address/0x8d10c03bc0889a2edea0de12e455a19ac7395b98)` contract for a bit, I learned that even for `1 ETH` the precision is not good enough, but if we’re willing to make `10 ETH`-sized transactions or larger, we’d actually observe a better price as predicted. The Bancor team recently updated the formula and mentioned it is a lot more accurate. Ultimately, I decided to skip re-doing this experiment with larger transaction sizes or the new formula in favor of the actual front-run.

### Hard Mode: trading automatically

Now, unless we are ready to sit in front of a computer all day and hit refresh on [etherscan.io](https://etherscan.io/?ref=hackernoon.com), the process needs to be automated. Luckily, most Ethereum clients provide a [JSON RPC](https://github.com/ethereum/wiki/wiki/JSON-RPC?ref=hackernoon.com) to interact with the blockchain and automate away the low-level details of interacting with the blockchain.

You just need to run a full node client and send API requests to `localhost:8545`.

  

  

  

  

$ sudo apt-get install software-properties-common$ sudo add-apt-repository -y ppa:ethereum/ethereum$ sudo apt-get update$ sudo apt-get install ethereum$ geth --rpc

Here is an example `curl` request that looks up a transaction by hash (in this case, a huge BUY order on Bancor):

Now let’s send the same request using Python:

If you got the same output as from `curl`, congratulations! The hardest part of learning to programmatically interact with the blockchain is already over.

Now implementing the front-running trader becomes a matter of putting together a few common API requests (pseudocode below for brevity). If the goal is to be maximally efficient, it is better to avoid cashing out in between transactions: front-running can be done in both directions, and it would be the most profitable to only sell once we see a pending sell, and only buy once we see a pending buy:

In my case I only wanted to prove that the idea works (and yields non-negligible amounts of money), so I did the front-run once and sold immediately. Better yet, in this case the “loser” of the trade is easy to pin-point (the only person losing money is the owner of the trade being front-ran), so it was easy to return that money too. More details to be found in the [full code](https://github.com/bogatyy/bancor?ref=hackernoon.com) on my GitHub.

The [results](https://etherscan.io/tokentxns?a=0xca83bd8c4c7b1c0409b25fbd7e70b1ef57629ff4&p=24&ref=hackernoon.com) of the front-run (transactions in reverse chronological order):

[](https://hackernoon.imgix.net/hn-images/1*sTUpW73PCEFf4ilytzoQlw.png "Download image")

![](data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20version=%271.1%27%20width=%271200%27%20height=%27800%27/%3e)![image](https://hackernoon.imgix.net/hn-images/1*sTUpW73PCEFf4ilytzoQlw.png?w=3840)

We made `0.477 ETH`, or a `~0.5%` return in less than a minute! I calculated the amounts (my own principal and the threshold for front-running) so that the profit would be at least $100, which I deemed convincing enough for the purposes of my post. Noteworthy, just a few days later there was a whopping `[5856 ETH](https://etherscan.io/tx/0x551137eb0558015aea760dbd8c8bfc1ee73308b427db547cb92329e3ab1dcc47)` [trade](https://etherscan.io/tx/0x551137eb0558015aea760dbd8c8bfc1ee73308b427db547cb92329e3ab1dcc47?ref=hackernoon.com) purchasing Bancor, which would have yielded an approximately `9%` return, or about $3000 given the same `100 ETH` principal!

The next part will explain where do these numbers come from and how to calculate the return from front-running a given trade.

### Simulations & ROI evaluation

Let us describe the two core assumptions behind the Bancor pricing system, and the exact formula that is derived from those assumptions. First, Bancor maintains a constant ratio between the **_traded token_** market capitalization and total **_reserve token_** value. As of right now, the only traded token is `BNT`, and the only reserve token is `ETH`. Assume there is approximately `70K ETH` in reserves (which is roughly the case) and the reserve ratio is `10%`. This means the whole market cap of the system is implied to be `700K ETH`, and the price per `BNT` is determined from this total market cap, divided by total `BNT` supply. If somebody buys `BNT` with `ETH`, the whole `ETH` amount is added to the reserves, thus pushing the price up (there was more value added to the reserves than tokens issued), increasing the value of everybody else’s tokens. Conversely, selling `BNT` reduces the reserve and thus the price per token (this way, the reserve never gets depleted, you just get less and less of it per token).

The second assumption is that a large trade should be equivalent to making a set of smaller trades of the same size (so to determine the final price, one would have to calculate an integral of `**_Price d(Size)_**`). Turns out these two assumptions are sufficient to uniquely define the behavior of the system in all cases. For those with a strong mathematical inclination, there is [proof](https://drive.google.com/file/d/0B3HPNP-GDn7aRkVaV3dkVl9NS2M/view?ref=hackernoon.com) available. Here are the resulting formulas:

[](https://hackernoon.imgix.net/hn-images/1*gWxTvemJfgdAXS17IxnW7Q.png "Download image")

![](data:image/svg+xml,%3csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20version=%271.1%27%20width=%271200%27%20height=%27800%27/%3e)![image](https://hackernoon.imgix.net/hn-images/1*gWxTvemJfgdAXS17IxnW7Q.png?w=3840)

For our practical purposes though, the exact formulas are not really necessary (and as we have learned, the actual implementation is not very precise anyway). Approximating the pricing formula with the linear part of its [Taylor’s series](https://en.wikipedia.org/wiki/Taylor_series?ref=hackernoon.com), we get:

`NEW_PRICE ~= OLD_PRICE * (1 + DEPOSIT / RESERVE_BALANCE)`

Interestingly, the `RESERVE_RATIO` is not a part of the approximation. I’ll skip the full derivations here, but basically, it cancels out in the numerator and the denominator, so the only thing that ends up mattering is `RESERVE_BALANCE = TOTAL_MONEY * RESERVE_RATIO`. This pricing formula is a good approximation as long as the deposit is much smaller than the total reserve. So if BNT has `70K ETH` in reserves, and somebody invests `700 ETH`, the price goes up by `700 / 70K = 1%`. If somebody withdraws `700 ETH` out of the system, the price drops `1%`. Thus in both cases front-running those transactions would give us instant `1% ROI`. **Note that our theoretical approximation matched practice quite well: a** `**350 ETH**` **trade yielded a** `**0.477%**` **return, against approximately** `**0.5%**` **predicted.**

Based on these calculations, I wrote some [code](https://github.com/bogatyy/bancor/blob/master/simulation.py?ref=hackernoon.com) to evaluate how much money could have been made front-running Bancor in July and August. Assuming we leave small transactions alone and only go after big ones (`> 100 ETH`) so that gas prices don’t really matter, we get:

  

  

  

  

  

$ python simulation.py...ROI for front-running all transaction >= 100 ETH:July 88.7%August 28.6%With a principal of 100 ETH, that would make you $35190

**Not bad: an attacker can more than double the money they invested into the attack over a couple months, chipping away from other Bancor users.** In practice, the profitability threshold is smaller than `100 ETH` (the total fees one would need to beat come out to a few dollars), though it also depends on the principal invested by the attacker.

### Front-running other Bancor-**exchangeable tokens**

While we did show that it was possible, `BNT` itself is relatively hard to front-run because of the large reserves: its price changes very little between transactions. But for any smaller token the `1 / RESERVE_BALANCE` fraction would skyrocket an attacker’s profits and rob honest investors very fast. To prove that, I have deployed my own token following the Bancor Protocol ([address](https://etherscan.io/address/0xf90a54b20881ece27e3a8e31903e3e75009a0164?ref=hackernoon.com), [code](https://github.com/bogatyy/bancor/blob/master/solidity/DummyBancorToken.sol?ref=hackernoon.com)) and made instant `2X` profit front-running a large transaction.

Our `FBT` (short for Front-Runnable Bancor Token) was initialized to have a total supply of `1M wei` (where `wei` is the smallest possible unit in Ethereum, equal to `10^-18 ETH` ), and the total initial supply of `2M` tokens. Given the `10%` reserve rate ratio, the implied market capitalization of our token becomes `10M wei` , and the price per token `10M / 2M = 5 wei` . Now what happens when the front-runner makes a deposit before a large “honest” BUY, but sells after?

[TX1](https://etherscan.io/tx/0x11cf84b2a98f5b8d72f5ba63b193aa78ba92d73cf63fa840a3f92d64bf97ee4f?ref=hackernoon.com), front-run:`1000 wei` gives `199 FBT` , in line with the price we calculated

[TX2](https://etherscan.io/tx/0x2296386f370ebf25d16a1f9f8bf7e35e20d6b9482786342939a399fb796eeb06?ref=hackernoon.com): very large buy, increasing the price roughly `2X`

[TX3](https://etherscan.io/tx/0x0109ae1b29fc24db0d18111f0c6581988a1ac58cfe94dded4f30578a6ea4100e?ref=hackernoon.com): attacker withdraws `199 FBT` for `1910 wei`, doubling their money

**Since the intended use of Bancor is to serve the lesser known tokens (which may not have enough demand and liquidity for regular exchanges), these tokens would naturally have smaller reserves, making this vulnerability especially dangerous. In an extreme case where the reserve is very low and an attacker has a lot of money, they can extract more value from an honest investor’s deposit than the investor would get themselves!**

Further, all of this is possible as a mere full node (that is, a person with a decent laptop). Miners, and especially miner pools, are in a privileged position and can do an order of magnitude more damage. Full-node attackers have to broadcast their transactions and thus risk their principal, whereas miners can mine blocks with their own front-run included, but never reveal it publicly unless they do mine the block. That way, they can profit at no risk or cost to themselves. Further, they can rearrange transactions within a block in whatever way they want, arbitrarily creating winners and losers out of other participants.

### Ethics

Like any new technology, this situation raises very interesting ethical questions. Is the strategy we discussed “hacking”? Is it high-frequency trading? Is it simply an ability to make informed decisions based on public information faster than other investors? Ultimately, is this kind of trading “bad”?

Personally, I took an easy way out and made the decision that lets me sleep the most soundly: returning the money after proving the point and stopping the program. Nevertheless, I would not have blamed someone in a similar spot if they decided to do otherwise.

### Solutions

Over the past month, [Haseeb Qureshi](http://haseebq.com/?ref=hackernoon.com) and I discussed several solutions with the Bancor team, making sure the vulnerability is contained. Yesterday, the team released a fix that handles most of the risk in practice. Long-term, theoretically robust solutions are fairly complex, and while analyzing those can be a whole different post, I want to briefly mention the options available.

**TL;DR don’t send large transactions and use** [**Bancor Web3 UI**](https://app.bancor.network/?ref=hackernoon.com) **that sets a** `**minReturn**` **for you.**

One partial solution is to set a `minReturn` on trades, basically canceling your order if you realize someone squeezed in front of you. This does prevent attackers from making guaranteed money instantly, but raises the question: what is a buyer supposed to do next? The order just revealed their intention to buy, and presumably they still want their Bancor, so they’ll place another buy order sometime in the near future, which will eventually raise the Bancor price and, on average, will profit the attacker just the same.

Yesterday, the Bancor team released a [Web3 interface](https://app.bancor.network/communities/5967699a4a93370018b7b891?ref=hackernoon.com) that implements the `minReturn` solution. Longer-term, and assuming perfectly intelligent adversaries, this might lead to some curious Nash equilibria (e.g. front-runners might block trades unless they are allowed a small profit margin, though a lower one than if they were front-running entirely naive users), but right now, this should solve most of the practical risk.

The Bancor team also suggested setting a universal `maxGasPrice`, to make sure non-miner front-runners cannot bid higher. This would fully protect the users from non-miner attacks (at a cost of lower liquidity during network congestion), although the original [front-runs by miners](http://hackingdistributed.com/2017/06/19/bancor-is-flawed/?ref=hackernoon.com) would not be affected. This fix will be out soon too.

More robust solutions would involve a version of the [commit-reveal](https://karl.tech/learning-solidity-part-2-voting/?ref=hackernoon.com) scheme, one of the go-to instruments in a cryptographer’s toolbox. [Submarine sends](http://hackingdistributed.com/2017/08/28/submarine-sends/?ref=hackernoon.com) by Cornell researchers proposes the most beautiful and general solution that I know of. However, given the specifics of the Bancor protocol, a much smaller solution could suffice: do a commit-reveal with a penalty for non-revealing in Bancor tokens themselves. Basically, whenever someone commits a hash of a trade and doesn’t reveal, a percentage of both their Bancor and ERC-20 Ethereum tokens is burned (note that it works irrespective of the direction of the trade, thus revealing no information). If designed with careful attention to detail (for example, making sure that reveals are only accepted for commits in a previous block), this can fully solve the front-running problem, including front-running by miners.

Of course, even the simplified scheme is so complicated that almost no one would want to perform it by hand in MyEtherWallet. As the ecosystem grows, more sophisticated client programs (like the auto-trader we just implemented, or the commit-reveal client) would have to take that role on behalf of the users. Again: people aren’t sending their own Telnet requests every time they want to browse the Web, so why should the crypto world be any different?

### Acknowledgements

This project started in collaboration with [Haseeb Qureshi](http://haseebq.com/?ref=hackernoon.com) and [Preethi Kasireddy](https://medium.com/@preethikasireddy?ref=hackernoon.com) at the [IC3 Ethereum bootcamp](http://www.initc3.org/events/2017-07-13-IC3-Ethereum-Crypto-Boot-Camp-at-Cornell-University.html?ref=hackernoon.com) under guidance from [Ari Juels](http://www.arijuels.com/?ref=hackernoon.com), [Iddo Bentov](http://www.cs.cornell.edu/~iddo/?ref=hackernoon.com) and [Phil Daian](https://pdaian.com/?ref=hackernoon.com). The post was rewritten considerably with massive help from Haseeb and [Nader Al-Naji](http://nadertheory.com/?ref=hackernoon.com), who is about to take over the world with [Basis](https://www.basis.io/?ref=hackernoon.com).

A disclaimer just in case: this is a personal project and it does not represent my employer or anyone else’s opinions. Since I worked on it in my spare weekends, the timeline ended up being very protracted, and I hope the attentive readers will forgive some small discrepancies (the Ethereum price may oscillate across the post, and some Bancor contract addresses had to be updated).