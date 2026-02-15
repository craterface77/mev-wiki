# The fastest draw on the Blockchain: Ethereum Backrunning

[

![Alex Manuskin](https://miro.medium.com/v2/resize:fill:64:64/1*hPq2YdYPJn0IP_yf5IopAg.jpeg)





](https://amanusk.medium.com/?source=post_page---byline--6bd19fabdbe1---------------------------------------)

[Alex Manuskin](https://amanusk.medium.com/?source=post_page---byline--6bd19fabdbe1---------------------------------------)

Follow

14 min read

·

Jul 22, 2020

728

[

](https://medium.com/plans?dimension=post_audio_button&postId=6bd19fabdbe1&source=upgrade_membership---post_audio_button-----------------------------------------)

Decentralized finances create an interesting dynamic of auctions. Arbitrage and liquidation opportunities are examples of auctions where the first player to make the bid can make a healthy profit. This creates a race where multiple bots spam the transaction pool, competing for a good spot in the block. It is interesting to witness these shootouts between bots and traders, and even more interesting to understand how it works. This dynamic of [backrunning,](https://github.com/ethereum/go-ethereum/issues/21350) while interesting, also creates a burden on the network, which has to process all transactions broadcasted by the bots.

A good example took place during the token launch of bZx on Uniswap. During the token launch, a lone shooter, operating a sophisticated bot, was able to pool off a cool profit of 0.5M USD in a few minutes. This report covers the technique that makes it possible, and how can anyone become the fastest gunslinger in the wild west of DeFi.

## An example of the BZRX token launch

With DeFi popularity on the rise, protocols operators are looking for ways to make a profit. Holding an ICO has fallen out of favor, and many companies rightfully chose to avoid this route. An interesting alternative is to launch a [governance](https://zengo.com/farming-comp/) token on a decentralized exchange in an IDO (Initial DEX offering). This allows the market to set the value, instead of a predetermined ICO launch price. bZx, a DeFi protocol (that has recently grabbed some [headlines](https://medium.com/@peckshield/bzx-hack-full-disclosure-with-detailed-profit-analysis-e6b1fa9b18fc)) announced their intention to launch a token on the Uniswap decentralized exchange on July 13th.

The auction began on block [10451767](https://etherscan.io/block/10451767), when bZx listed their brand new token on Uniswap, [providing](https://etherscan.io/tx/0xd5cf62c1b41beb43ca1c7a05cf348300db0d7e1c8ac0d35041bbc45367811a5c) 5M BZRX. Immediately after that, a trader has [bought](https://etherscan.io/tx/0x94e644e6988b9229db0effad7cca2e9864f4173ee9b01e5a3e25540305206f46) almost half of the freshly minted BZRX tokens at a slightly higher rate.

As more and more participants bought the token, the price of BZRX increased.  
A few minutes after the initial purchase, the [trader](https://etherscan.io/token/0x56d811088235f11c8920698a204a5010a788f4b3?a=0x0ea72bf9aae7bb9e8eb97c965d443e38488cd546) started selling BZRX at much higher prices, ~10x the initial listing. After selling almost all the tokens back to Uniswap, the trader gained a cool ~550K USD in profit. All within a few minutes.

To achieve this, the winning trader applied careful preparation including a smart contract, an array of ~700 Ethereum accounts, and clever monitoring of the transaction pool.  
The technique used to accomplish this trade can sometimes be seen in events that benefit being the _first_ to execute a transaction, such as closing an arbitrage opportunity or [liquidations](https://zengo.com/defi-research-understanding-compound-liquidators/).

Transactions to close arbitrage are not a new concept of course and exist in traditional markets as well. What makes them special in the blockchain space, is that blockchains are completely transparent and transactions on the chain can be uniquely serialized to reflect the true unfolding of events. There can be no dispute over who made the winning trade first once the dust settles. So, what makes blockchain transactions so special?

## You own the blockchain

There is a nice property about blockchains, there is only a single truth (leaving forks out of the discussion). When a transaction is mined on the Ethereum blockchain, it has a unique mandate to alter the state of the chain. Every transaction happening before it, and after it, will see a different state. To know what was the truth at a certain point in time, simply playback the “tape” of the blockchain, block by block, and transaction by transaction.

==Thus, when a transaction is included in a block, it does not only perform its function, it also grabs the only lock that exists to the blockchain’s state. At the right opportunity, acquiring this lock might be very lucrative, and thus very expensive.==

## A lottery example

Assume uncle Bob is holding a lottery. The first person to pay Bob 1 ETH, will receive 10 ETH from Bob.  
Bob announces the start of the lottery on a certain date and time. To mark the start of the lottery, Bob broadcasts a special transaction to the blockchain, henceforth, the _lottery_ _transaction_.

The rules to win Bob’s lottery are as follows:

- Anyone can participate
- The first transaction to pay Bob 1 ETH **after** the lottery transaction, wins
- Any transaction paying Bob 1 ETH before the lottery transaction, gets nothing
- Any transaction paying Bob 1 ETH after the winning transaction, gets nothing

Bob does not know at which exact block his transaction will be mined, and at what order within the block. This is for the _miners_ to decide.  
Only the lucky transaction to be picked exactly after the start of the lottery will win, all transactions before and after it will fail. It is the _only one_ to fit the conditions to succeed. Additionally, no other transaction can come between the start of the transaction, and the end of its execution.

## Transaction ordering

Before getting to how a smart player can win this lottery, we take a step back and look at how blocks are ordered.

Ordering transactions in a block is done by the miners. While miners look for a solution to the proof of work puzzle, new transactions are being added to the transaction pool (Mempool in bitcoin terminology). This transaction pool is all the transactions the miner can choose from to construct a block. These are the pending transactions, waiting to be mined.

Each transaction carries a fee. The fee is a payment to the miners, to encourage them to include your transaction over someone else’s. As demonstrated in the figure, the space in the block is limited, so if you want to get in, you have to make your transaction more appealing.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/0*u8sbGgBnQPgh1GZz)

Miner ordering transactions in a block

Generally, miners will include the more lucrative transactions first, these will usually appear towards the start of the block. Transactions go down in price towards the end of the block, as miners have less lucrative transactions to choose from.

This order is not mandatory but is often used as a basic transaction sorting algorithm. There is no clear way to order transactions at the same price. A miner is free to choose whatever order they want (e.g. fist come first served).

This ordering implies that transactions being picked first will also “happen” first when playing back the tape of the blockchain.

## Buying the winning ticket to Bob’s lottery

With this knowledge, a naive strategy to try and win Bob’s lottery could be:

1. Watch the blockchain
2. Wait for Bob’s lottery transaction to be included in a block
3. Send a very high paying transaction to be the first transaction in the block **following** the block where Bob’s transaction gets mined.

This strategy is good, but not optimal. Entering one block after Bob’s transaction might be too late. Other transactions can come after Bob’s transaction **within the same block**_._ The player needs to be a real sharpshooter to draw their transactions as soon as Bob’s transaction gets mined and hit in just the right spot.

## Increasing the odds

There may be many players trying to win Bob’s lottery. A simple way to increase the odds of winning a lottery is to fill multiple tickets. In this case, send _multiple_ transactions.

For this example, assume that any transaction that does not win the lottery simply fails, and does not lose the 1 ETH. This reduces the cost of participation. Sending multiple transactions is still not free though, transactions must pay gas fees, and if gas prices are high, and the transaction is complicated, this can get quite expensive.  
A player willing to participate needs to consider these costs. A player might decide it is worth it to participate in this lottery despite the added costs. That is, even if they send 100 transactions, the chance to profit 10 ETH outweighs the risk and the associated costs.

How could the player send 100 transactions at once? A single user in Ethereum is bound by a sequential counter, the _nonce_. The nonce keeps track of transactions made by a single address, no transaction with nonce _n_ can be mined if all transactions leading up to nonce _n_ have not been mined yet. So to send 100 transactions simultaneously, of 1 ETH each, the player needs 100 accounts, all preloaded with 1 ETH, to all attempt to send Bob a transaction at once. They need 100 ETH to gain 10, not optimal. Enter smart contracts.

## Smart shooting

Smart contracts can be programmed to execute any action desired. For example, paying 1 ETH to Bob. If the participating smart contract is the first to pay 1 ETH to Bob, this smart contract will be the one winning the lottery.

==The cool thing is that a smart contract does not have a binding nonce, the player can send multiple transactions, from multiple accounts to their self owned smart contract.==

The solution is thus to preload the smart contract with 1 ETH, and have 100 addresses with minimal amounts, only to cover the gas fees of invoking the smart contract. At the right time, broadcast 1 transaction from each account to the smart contract. The smart contract will pay Bob, and win the lottery. Then, just collect the funds from the contract to one of the accounts.

With some knowledge of smart contracts and meticulous planning, a player can significantly increase their chances of winning the lottery, while only using 1 ETH (and some change).

Going back to the fastest gunslinger analogy, if using 1 account with 1 ETH is like pulling out a gun, using smart contracts is like pulling out 100 guns at once, without overpaying for the bullets.

![](https://miro.medium.com/v2/resize:fit:640/0*fY1WyDVJlSmv3hOm)

## Sniping a transaction in a block

Now there is the question of timing. A sub-optimal strategy is to blindly fire transactions near the time specified by Bob. This could quickly become very expensive.  
Even if Bob is true to his word, he might not broadcast the lottery transaction at just the right time. There may be time differences, network delays, etc.

What would be a better strategy to spot the exact moment when Bob’s lottery transaction is mined? It’s all about monitoring the transaction pool.  
The transaction pool offers a “glimpse into the future” of the blockchain. The order is not determined, but the pending transaction in the pool will eventually find their way into a block (if they are valid).

Anyone can parse and inspect these pending transactions. The strategy to get a transaction just in the right place is thus as follows:

1. Watch the transaction pool
2. Notice the special transaction (e.g. Bob’s lottery)
3. Fire multiple transactions with **the exact same gas price** as Bob’s transaction.

### Why the same price?

Exactly because the miners are picking transactions mainly based on their gas price, by sending multiple transactions with the same gas price, there is a high chance that the winning transaction will end up close to the lottery transaction.

Some might fall before it, some may be after it. As long as the winning transaction falls right after Bob’s, it wins the lottery.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/0*jZzFhHd2iLABvVXN)

Only one winning transaction

This type of transaction hunting is often called “[sniping](https://en.wikipedia.org/wiki/Auction_sniping)”. Getting a transaction just in the right slot. The name is a bit misleading though, as what actually happens here better resembles carpet bombing or machinegun fire.

![](https://miro.medium.com/v2/resize:fit:1280/0*XuCnO2OObrjkhR4e)

An interesting point here is that paying more for gas fees does not actually yield better results. The higher paying transactions will be slotted towards the start of the block, and will not meet the conditions for a win (Bob’s transaction hasn’t happened yet). As a side effect, this hunt for a good spot in a block creates a lot of “spam”. The vast majority of these transactions will fail without executing anything.

## The shootout

For a real-world case study, let’s return the bZx token launch. The DeFi platform for lending and borrowing, bZx announced their intentions to launch their own new token via an IDO on Uniswap. News of the launch was [public](https://twitter.com/defiprime/status/1282397341064945664) knowledge.

To create a market on Uniswap, a liquidity provider(in this case bZx) supplies both ETH and tokens. The ratio between ETH and tokens sets the initial price of the token. From that point on, buying tokens from the market increases the price, selling tokens on the market decreases the price.

[](https://medium.com/write?source=promotion_paragraph---post_body_banner_jsw_blocks--6bd19fabdbe1---------------------------------------)

The traders’ strategy was as follows:

1. Swoop in and buy as many tokens as possible right after the IDO launches (but not all of them as there needs to be an opportunity for others to buy tokens as well)
2. Wait for the price to go up as other traders buy the token from Uniswap
3. Sell back the tokens at a higher price

The key is to have the **first** transaction be slotted immediately after the launch. The first transaction to make the trade gets the best rates, and since the token is available **only** on Uniswap, any trade would be a buying trade which increases the price of the token.

To execute this, the trader followed the steps discussed for winning Bob’s lottery:

- Set up a [contract](https://etherscan.io/tx/0x9f87153c02bc31da11d45f5c0acb95df5c3ee5492f575880c2a03bd689d5d81a)
- [Preload](https://etherscan.io/tx/0x1cda1408bdf274edf7b5409cee6dd8e63e06b33802cfbf4db745303858bd2efd) the contract with 650 ETH to buy the tokens
- Prepare multiple addresses with funds for gas
- Broadcast all transaction as close to the IDO as possible

The trader’s contract was launched on July 13th, 2020 12:50:17 PM +UTC.

The market for bZx token (BZRX) at the center of the BZRX token was [launched](https://etherscan.io/tx/0xd5cf62c1b41beb43ca1c7a05cf348300db0d7e1c8ac0d35041bbc45367811a5c) on Uniswap on July 13th, 2020 02:28:24 PM +UTC, on block [10451767](https://etherscan.io/block/10451767), supplying 1000 ETH and 5M BZRX tokens, setting the price at 0.0002 ETH/BZRX.  
The trader’s contract was actually created less than an hour before the expected token launch.

To further appreciate the effort of the trader, it’s interesting to look at how many addresses were used to send transactions to the smart contract at the launch of the market on Uniswap. This number amounts to [732](https://explore.duneanalytics.com/queries/6183/source#12257) Addresses!  
That is, besides writing and launching the smart contract for this specific task just in time, the trader also funded at least 732 addresses with ETH. All launching a transaction in just the right time to try and capture the slot that brings in the most value.

The plan worked! One of the trader’s transactions was slotted exactly one transaction after the IDO launch. The image shows all transactions preceding the IDO launch fail. Transactions following the “winning” transaction also fail. All transactions have the trader’s smart contract as a destination (0x0ea..).

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/0*kXzU4RObzayIrpJd)

Sniping the IDO transaction ([source](https://etherscan.io/txs?block=10451767&p=3))

The winning trader was actually not alone, several other contenders were attempting to pull off the same strategy, launching a barrage of transactions of their own:

- [Bot1](https://etherscan.io/txs?a=0xb366189fdedac44184905ec7ce070dfa9e75a63d) (59 Txs in block [10451767](https://etherscan.io/block/10451767))
- [Bot2](https://etherscan.io/address/0xb30eb6daffb8a2b09eaa6a0ac5f6af16b0c90e82) (23 Txs in block [10451767](https://etherscan.io/block/10451767))

The winner ended up being the one putting in the most effort, with 141 out of the 266 transactions in the block.

All bots followed a similar strategy, out of 266 transactions in the block, 232 had the same gas price as the IDO [transaction](https://etherscan.io/tx/0xd5cf62c1b41beb43ca1c7a05cf348300db0d7e1c8ac0d35041bbc45367811a5c), 60 GWei exactly.

## Collecting the profit

Getting the right transaction in place was only half the work. The trader also needed to dump the tokens in just the right time to gain a profit.

A short time after the trader bought up almost half of all BZRX tokens in the initial Uniswap pool, the price spiked up, as additional traders entered the game.

Only 14 blocks from the start of the IDO, the trader started selling the BZRX back into the pool, at much higher rates than the initial purchase of course.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/0*90yIaFK7scXpLGX1)

BZRX price shortly after the IDO

The table summarized all transactions made by the winning trader, from the initial purchase, until almost all their BZRX were sold back.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/0*65gNpDTvDPPAzGce)

All trades made by the winning bot ([source](https://etherscan.io/token/0x56d811088235f11c8920698a204a5010a788f4b3?a=0x0ea72bf9aae7bb9e8eb97c965d443e38488cd546))

The total profit from the trade: ~**500K** USD in ETH and another ~**40K** USD in BZRX tokens.

The total gas costs for broadcasting all transactions to the contract, sum up to 1.47 ETH, or ~350 USD. Boom!

![](https://miro.medium.com/v2/resize:fit:1000/0*cU66AuQLwfyRDO73)

## Risky business

This example is very impressive, trading $350 of gas fees for $0.5M is a great deal, however, as always, it is also important to consider the risks.

If some other trader had gotten the first transaction, all transactions of this trader would have probably failed, profiting only miners.

There is the risk that no one actually buys the tokens, the price does not go up and the trader gets stuck with bags of worthless tokens.

Additionally, writing smart contracts is not foolproof. An error in the contract might result in having funds locked up, or lost forever. This requires skills, testing, and preparation. It is probably not the first time the trader pulled something like this off.

Last but not least, the trader had to come up with the initial capital to buy up the liquidity. 650 ETH is not a small amount to play games with.

This special combination of skills, funding, and risk-taking, all came together to yield this impressive result but could have also ended sourly.

## Inside knowledge?

Some specifics about the trade do raise the question of whether or not there has been some insider information involved here. Mainly, how did the trader know how much liquidity is going to be provided to the market in the first place?

![](https://miro.medium.com/v2/resize:fit:688/1*_DvTzASlMVkT5f8T9yZxVA.png)

The first transaction, setting the price, provided 5M tokens, and 1000 ETH into the liquidity market. The trader bought up 650 ETH worth of tokens, leaving enough in the pool for the following trades to increase the price.  
What’s interesting is, the all transactions to the smart contract of the trader did not carry any parameters, suggesting that the values for how many tokens to buy, and the slippage rate were hardcoded into the contract.  
If the market would have been launched with only 500 ETH of liquidity, all the trader’s transactions would have failed.

Besides, the fact that the contract was only uploaded 1 hour before the trade might also be suspicious, although the timing of the start of IDO was known in advance.

## The upper hand of the miners

Before concluding, there is an elephant in the room that needs to be discussed, and that is the miners themselves. This trade, and many like it, rely on the miners to fairly include transactions in blocks. This however does not have to be the case. Miners have the ultimate upper hand in this game, should they so desire.

A miner chosen to mine the IDO start transaction can censor all other transactions in the block, and only include their own transaction, performing exactly the same trick. They are in control of what goes into the block and in what order.

We rarely see miners taking full advantage of this capability. Perhaps the profits from such activities are not lucrative enough for miners to engage at the moment.

This is part of the delicate system of incentives each blockchain has in place. At the moment, miners are happy to pick up the extra fees associated with these kinds of “sniping” trades. However, if block rewards become too low, it is very well likely miners will up their game and start front running transactions on an entirely different scale.

## Final thoughts

Some might argue the IDO launch was unfair, and that the winning trader had an unfair advantage. Sure, it does appear unfair that one player takes away profit that could have gone to bZx themselves, or their token holder, however, this was not given away for free.

The trader had to:

- Know of the IDO launch
- Take a risk on the price going up significantly
- Prepare a trading smart contract
- Load up 700 addresses with funds to all fire transitions at once
- Monitor the transaction pool
- Correctly and promptly spot the right transaction
- Simultaneously broadcast 700 transactions
- Spend ~350$ on fees that might have been lost
- Quickly convert all purchased tokens into profit

In my opinion, this is a hard day’s work. Also, it is hard to argue unfairness when others could and _did_ participate in the same auction. The winner ended up being the one most committed and most prepared. For the future, platforms might consider other ways to launch tokens.

An interesting way to look at these events is that what all these bots were actually bidding for, is just the right spot inside the block. The way to win this spot requires knowledge funding and some risk-taking. As DeFi becomes more and more interconnected and complex, grabbing a slot in a block, especially if it is a very good one, is going to become increasingly expensive.