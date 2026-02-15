By: Eyal Markovich, Co-founder & COO

## The Fear of MEV (Miner Extractable Value)

For most Ethereum users the word MEV can be terrifying; MEV is associated with front-running, sandwich attacks and other attacks that result in one user profiting at another user’s expense.

If a user swaps tokens on an AMM (e.g. Uniswap) for large enough value, there is a good chance that transaction will be frontrun.

If a user swaps tokens on an AMM that creates slippage, there is a good chance that transaction will be backrun.

There are, however, several MEV use cases that can benefit the average user. In this post I want to share such a use case and introduce a new service called [BackRunMe](https://backrunme.com/)

[BackRunMe](https://backrunme.com/) is a service that allows users to safely submit private transactions (e.g. protection against frontrunning and sandwich attacks) while allowing searchers to backrun the transaction via MEV if it produces an arbitrage profit. _Most importantly BackRunME, gives a portion of this additional profit back to the user_.

## Understanding the MEV Ecosystem

A common frontrunning attack involves three players — the victim user being frontrun, the trading bot, and the miner who mines the block which includes the frontrunning attack. BackRunMe leverages all these players to offer a service to benefit the average user.

### The User

Users must get their transaction (e.g. a token swap) to miners to be included in the next block. If a user submits a transaction via a global infrastructure (e.g. MetaMask which uses Infura by default), this transaction will traverse node to node through the Dark Forest (the network) until it gets to miners. Waiting in the Dark Forest are bots that are looking to frontrun or sandwich attack such transactions.

To avoid being frontrun or sandwiched, users can use services that limit slippage or use private transactions to travel to miners undetected by bots. Indeed, bloXroute offers [Frontrunning Protection](https://docs.bloxroute.com/apis/frontrunning-protection) with direct private communication to mining pools.

However, some transactions also create opportunities to make additional profit that do not harm the user that sent the transaction. For example, a Uniswap transaction that creates slippage can generate an arbitrage opportunity where backrunning the transaction can capture additional profit. In this case, for the most part, the user that submitted the transaction, is not affected by the backrun transaction (since it occurs after the transaction is confirmed).

By now, you might be able to get a sense of where this is going.

### The Bots (Searchers)

These sophisticated bots scan the Ethereum mempool to identify any sandwich, arbitrage and other profitable opportunity. Many of the bots operators are using bloXroute’s fast [data streaming service](https://docs.bloxroute.com/streams) to hear about new transactions in the mempool. With the rise of MEV, these bots can create bundles (e.g. a group of transactions made up of the bot’s transaction plus a user’s trigger transaction) and submit it to mining pools via MEV services like Flashbots and [bloXroute](https://docs.bloxroute.com/apis/mev-solution).

Bundles submitted via such channels compete by paying higher fees (tip) to miners and eventually the bundle that pays the highest price will be selected by the pool.

What about transactions that are submitted to bloXroute’s Frontrunning Protection (e.g. private transactions)? These transactions are sent directly to pools and are not accessible to bots, thus users are protected from frontrunning attacks. However, that is also true for transactions that create arbitrage opportunities via a backrun transaction (which as discussed above does not affect the user). These transactions are also hidden from arbitrage bots.

Now, you should know where this is going.

### The Pools

As mentioned above, the role of miners (e.g. pools) is to put transactions into blocks and add these blocks to the chain. Pools use bloXroute to propagate their blocks faster in order to reduce their uncle rate. In addition, pools receive private transactions via private communication and MEV bundles from bloXroute to be added to the block. To do this, pools directly connect to the bloXroute BDN.

## BackRunMe

BackRunMe is a service that allows users to safely submit private transactions (e.g. protection against frontrunning and sandwich attacks) while allowing searchers to backrun the transaction via MEV if it produces an arbitrage profit. _Most importantly BackRunMe gives a portion of this additional profit back to the user_.

BackRunMe combines the arbitrage bot’s technology with MEV and frontrunning protection to create a win-win scenario.

[](https://medium.com/plans?source=promotion_paragraph---post_body_banner_dot_calm_field--2d7b77f4ca2d---------------------------------------)

Profit sharing structure:

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/0*picOBGHVPIERqRTE)

If a transaction is not creating a backrun opportunity — it will be processed as a normal private transaction.

![](https://miro.medium.com/v2/resize:fit:1400/0*7RxEwO-JMO3zudeI)

### How Does it Work?

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/0*kcMK7UmwCYKjBanl)

### What protects users from being frontrun?

Searchers only get metadata info about the private transaction and do not have the signed raw transaction. Thus, they cannot submit a regular MEV bundle without having the signed private transaction. bloXroute will only create an MEV bundle if the searcher proposed transaction is a backrun transaction that is paying the user 40% of the profit.

## How to use BackRunMe

You can use MetaMask directly on [backrunme.com](https://backrunme.com/) trade on Uniswap or Sushiswap. If you’d like to use MetaMask + BackRunMe anywhere else, configuring MetaMask’s RPC allows you to bring it anywhere on Web3.

**BackRunMe Website**

To trade on Uniswap V2 and Sushiswap, use the BackRunMe user interface at [backrunme.com](https://backrunme.com/).

![](https://miro.medium.com/v2/resize:fit:982/0*yd8Ac4ipFjZcOFE8)

### Metamask custom RPC

You can use MetaMask directly on [backrunme.com/](https://backrunme.com/) to trade on Uniswap or Sushiswap. If you’d like to use MetaMask + BackRunMe anywhere else, configuring MetaMask’s RPC allows you to bring it anywhere on Web3.

Follow the steps below to use BackRunMe service using MetaMask.

**Setup a Custom RPC**

To use BackRunMe with Metamask, users need to configure a custom RPC endpoint. Use [https://portal.bloxroute.com/private-transaction](https://portal.bloxroute.com/private-transaction) to onboard in a few minutes

Below are the steps to add custom RPC endpoint with a MetaMask wallet:

1. Open the “**Settings**” menu and tap on “**Networks**”. Then tap on “**Add Network**” on the Networks menu.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/1*WG-Q1s_0lZ0RwbOHOjIiHQ.png)

2. Fill in the the field as shown and click “**Save**”.

- **Network Name**: bloXroute Private TX
- **New RPC URL:** https://metamask-rpc.blxrbdn.com/
- **Chain ID:** 1
- **Currency Symbol** (optional): ETH

3. Then select the new network (bloXroute Private Tx) from the Network List.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:1400/1*alusjReLHZLOO9LM-u2sXQ.png)

**You’re ready to Submit a Private Transaction with BackRunMe**

This will allow you to avoid frontrunning and allow searchers to backrun you for additional profit.

### Questions?

Email to [support@bloxroute.com](mailto:support@bloxroute.com) for help.
