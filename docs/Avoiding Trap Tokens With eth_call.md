The JSON-RPC method `eth_call` (documentation [HERE](https://geth.ethereum.org/docs/rpc/ns-eth#eth_call)) is quite powerful, but rarely discussed. We will explore a very useful way to use `eth_call` here.

Before continuing, I recommend watching this video from notable MEV senpai [libevm](https://twitter.com/libevm):

The video will not make you an `eth_call` expert, but it does give a very nice overview of why `eth_call` exists and what you can do with it.

For background, I’ve been extending my payload executor / Flashbots project to support multi-pool (triangle) arbitrage. As a result, I’ve encountered a lot of really bizarre ERC-20 tokens.

[

BowTiedDevil | Code Chad@BowTiedDevil

Watching the Ethereum mempool for a few days Shook about the volume of $20 meme coin swaps Casino vibes fr fr

6:25 PM · Aug 17, 2022

---

9 Likes





](https://twitter.com/BowTiedDevil/status/1559939599526285317)

The issue with these bullshit ERC-20 tokens is that they often implement fees on transfer, have white/grey/blacklists that restrict transfers to and from certain address, and can be arbitrarily paused. At first glance this seems OK, since I’m doing atomic arbitrage and do not intend to hold KEANUINUDOGE long-term, but there’s a more subtle issue.

Often these arbitrage pathways look very profitable (calculated on good faith), but revert when simulated. My bot has no sense of a token’s “legitimacy” when comparing arbitrage pathways, and it cannot reasonably determine whether a potential arbitrage path is likely to fail.

I can’t hand-craft every arbitrage path or whitelist every token input. I don’t have the time to pick through tens of thousands of token contracts on Etherscan, and neither do you.

What’s the solution? My brothers in Christ, it’s `eth_call`!

## Simulating Token Transfers

To begin, let’s take an example of a bullshit ERC-20 token: [NICE](https://etherscan.io/address/0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88). From the contract comments:

```
// SushiToken with Governance.
contract NiceToken is ERC20("NiceToken", "NICE"), Ownable {
    // START OF NICE SPECIFIC CODE

    // NICE is a copy of SUSHI https://etherscan.io/token/0x6b3595068778dd592e39a122f4f5a5cf09c90fe2
    // except for the following code, which implements 
    // a burn percent on each transfer. The burn percent (burnDivisor) 
    // is set periodically and automatically by the 
    // contract owner (PoliceChief contract) to make sure
    // NICE total supply remains pegged between 69 and 420

    // It also fixes the governance move delegate bug
    // https://medium.com/bulldax-finance/sushiswap-delegation-double-spending-bug-5adcc7b3830f
```

Some joker has decided that the SUSHI token needed to be forked with a burn method that pegs the supply between 69 and 420. That’s kind of funny but my bot is very serious and has a poor sense of humor and yells a lot.

Here’s the transfer function that burns the fee portion prior to sending the remainder:

```
function transfer(address recipient, uint256 amount) public virtual override returns (bool) {
        // calculate burn amount
        uint256 burnAmount = amount.div(burnDivisor);
        // burn burn amount
        burn(msg.sender, burnAmount);
        // fix governance delegate bug
        _moveDelegates(_delegates[msg.sender], _delegates[recipient], amount.sub(burnAmount));
        // transfer amount minus burn amount
        return super.transfer(recipient, amount.sub(burnAmount));
    }

    // we need to implement our own burn function similar to 
    // sushi's mint function in order to call _moveDelegates
    // and to keep track of totalSupplyBurned
    function burn(address account, uint256 amount) private {
        _burn(account, amount);
        // keep track of total supply burned
        totalSupplyBurned = totalSupplyBurned.add(amount);
        // fix governance delegate bug
        _moveDelegates(_delegates[account], address(0), amount);
    }
```

My options:

- Implement a NICE-specific method that queries the contract for the current burn rate, recalculate the true swap amounts, and re-evaluate the arbitrage opportunity (terrible waste of time)
    
- Blacklist the token entirely (efficient waste of time)
    
- Test the token transfer using `eth_call` and eliminate it if any fuckery is detected (gigabrain move)
    

Let’s test the token on the console, then review how we might accomplish this in an automatic way. Start a Brownie console connected to Ethereum mainnet (either fork or live, it does not matter):

```
>>> token = Contract.from_explorer('0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88')
Fetching source of 0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88 from api.etherscan.io...
```

Now let’s run a very simple test of `eth_call` using Brownie’s built-in `call()` method on the `transfer()` function call. First generate a fake account, then attempt to transfer 100 NICE tokens from the official contract to the new account.

```
>>> account = accounts.add()
mnemonic: 'tunnel salad direct disease disease educate voyage slab cricket unable hip winner'
>>> token.transfer.call(account, 100, {'from':token.address})
True
```

Cool, that worked. But the problem is that the only information we know is that the transaction would have succeeded. It does not tell us how many tokens were actually transferred, and there’s no clear way to tell without running this on a fork.

Let’s do that just for educational purposes, so you can see how this token actually operates:

```
(.venv) devil@hades:~/bots$ brownie console --network mainnet-fork
Brownie v1.19.1 - Python development framework for Ethereum

BotsProject is the active project.

Launching 'ganache-cli --chain.vmErrorsOnRPCResponse true --wallet.totalAccounts 10 --hardfork istanbul --fork.url https://rpc.ankr.com/eth --miner.blockGasLimit 12000000 --wallet.mnemonic brownie --server.port 6969 --chain.chainId 1'...
Brownie environment is ready.

>>> token = Contract.from_explorer('0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88')
Fetching source of 0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88 from api.etherscan.io...

>>> account = accounts.add()
mnemonic: 'dog bulb screen idea salmon happy inside erupt message sniff notice image'

>>> tx = token.transfer(account, 100, {'from':token.address})
Transaction sent: 0x588fa08d32a11534ea323f1009c11c44e43ab566ae58379d5b56050811b3c04c
  Gas price: 0.0 gwei   Gas limit: 12000000   Nonce: 1
  NiceToken.transfer confirmed   Block: 15447914   Gas used: 71740 (0.60%)

>>> tx.info()
Transaction was Mined 
---------------------
Tx Hash: 0x588fa08d32a11534ea323f1009c11c44e43ab566ae58379d5b56050811b3c04c
From: 0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88
To: 0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88
Value: 0
Function: NiceToken.transfer
Block: 15447914
Gas Used: 71740 / 12000000 (0.6%)

Events In This Transaction
--------------------------
└── NiceToken (0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88)
    ├── Transfer
    │   ├── from: 0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88
    │   ├── to: 0x0000000000000000000000000000000000000000
    │   └── value: 10
    └── Transfer
        ├── from: 0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88
        ├── to: 0x7761270B4dd10cfac3E813bb47a635e68E941581
        └── value: 90

>>> token.balanceOf(account)
90
```

It’s clear that this is a fee/tax token, and you can see 10% of the transfer being sent to the zero address. Send 100, receive 90. Ultrasound money!

## Live Testing

The issue with this approach is that every time you want to test a token, it requires an isolated process and the use of a local fork. This can be automated, but it’s quite slow.

Lucky for us, `eth_call` allows us to do **on-demand testing** of tokens against a live network!

Unlucky for us, it’s really complicated…

To get the hang of using `eth_call` to its full potential, I’ll layer the complication on in small steps.

We used the native Brownie method `.call()` above to simulate a state-changing transaction. Brownie’s .call() implementation is a simple wrapper over the [web3py Eth.call method](https://web3py.readthedocs.io/en/stable/web3.eth.html?highlight=eth_call#web3.eth.Eth.call), so we can treat them as functional equivalents. The major difference is that web3py’s implementation allows you to submit an `eth_call` to an arbitrary address, where Brownie’s wrapper only runs off a contract object.

The web3py method requires one argument, `transaction`, which is a dictionary of relevant transaction parameters. The required parameters are:

- `from`
    
- `to`
    
- `data`
    

The optional parameters are:

- `gas`
    
- `gasPrice` (for type 0 transactions)
    
- `maxFeePerGas` / `maxPriorityFeePerGas` (for type 2 transactions)
    
- `value`
    
- `nonce`
    

### Transaction Example

We will build a transaction dictionary with all necessary parameters to access the `transfer()` method on the NICE contract.

We will build the transaction dictionary using web3py. The `data` parameter can be built three ways:

### 1. Brownie encode_input

```
>>> nice.transfer.encode_input(account, 100)
'0xa9059cbb000000000000000000000000071437d6919f75c7d40363aacd26ac7df39c71fe0000000000000000000000000000000000000000000000000000000000000064'
```

Brownie’s `encode_input` method allows you to generate calldata for a particular function from a contract object.

### 2. Web3.py build_transaction

```
>>> import web3
>>> w3 = web3.Web3(web3.WebsocketProvider())
>>> w3.eth.contract(
    address=nice.address,abi=nice.abi
    )
    .functions.transfer(account.address,100)
    .build_transaction({'gas':1_000_000})

{
    'chainId': 1,
    'data': "0xa9059cbb000000000000000000000000071437d6919f75c7d40363aacd26ac7df39c71fe0000000000000000000000000000000000000000000000000000000000000064",
    'gas': 1000000,
    'maxFeePerGas': 27474079452,
    'maxPriorityFeePerGas': 1000000000,
    'to': "0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88",
    'value': 0
}
```

### 3. ABI Encode With eth_abi

```
>>> import eth_abi
>>> w3.keccak(text='transfer(address,uint256)')[:4].hex()+eth_abi.encode_abi(['address','uint256'], [account.address, 100]).hex()
'0xa9059cbb000000000000000000000000071437d6919f75c7d40363aacd26ac7df39c71fe0000000000000000000000000000000000000000000000000000000000000064'
```

### Three Ways, One Result

You’ll notice that the generated calldata is identical for each of the three options. This is exactly as intended, and you should have expected it. If not, visit the lesson [Generalized Vyper Smart Contracts](https://degencode.substack.com/p/generalized-vyper-smart-contracts) to get familiar with function selectors and calldata.

## Accessing and Modifying Storage

This section will likely seem like a curveball, but it’s necessary to understand before you can really explore the state override functionality of `eth_call`.

All of the data inside a smart contract is stored at a particular address within the EVM. We take it for granted that data can be read at any time, but mostly because we’re used to contracts that expose that data via read-only view methods.

What if we want to modify that data for testing purposes? What if the smart contract author did not provide a view function?

This is where the `eth_getStorageAt` method comes in. Recall that EVM is a public blockchain and should be readable by anyone. The data may be obscured or encoded, but it cannot be hidden.

Our NICE contract, for example, includes several variables before the constructor:

```
contract ERC20 is Context, IERC20 {
    using SafeMath for uint256;
    using Address for address;

    mapping (address => uint256) private _balances;

    mapping (address => mapping (address => uint256)) private _allowances;

    uint256 private _totalSupply;

    string private _name;
    string private _symbol;
    uint8 private _decimals;

    /**
     * @dev Sets the values for {name} and {symbol}, initializes {decimals} with
     * a default value of 18.
     *
     * To select a different value for {decimals}, use {_setupDecimals}.
     *
     * All three of these values are immutable: they can only be set once during
     * construction.
     */
    constructor (string memory name, string memory symbol) public {
        _name = name;
        _symbol = symbol;
        _decimals = 18;
    }
```

We will start slow, but the goal is to learn how to access the variables `_balances` and `_allowances`, both of which are marked `private`.

Storage in a Solidity contract is allocated sequentially to EVM by order of declaration in “slots”. The first variable `_balances` is stored at slot 0. The second variable `_allowances` is stored at slot 1.

Mapping are difficult to work with, so we’ll skip to the next three variables:

- `_totalSupply` (slot 2)
    
- `_name` (slot 3)
    
- `_symbol` (slot 4)
    

Open a Python console, generate a `web3` object to use, then fetch the storage at slots 2, 3, and 4:

```
>>> import web3
>>> w3 = web3.Web3(web3.WebsocketProvider())
>>> _totalSupply = w3.eth.get_storage_at(nice.address,2)
>>> _name = w3.eth.get_storage_at(nice.address,3)
>>> _symbol = w3.eth.get_storage_at(nice.address,4)
```

Now inspect the contents:

```
>>> _totalSupply
HexBytes('0x00000000000000000000000000000000000000000000000fd17be9b4f9ce23e2')
>>> _name
HexBytes('0x4e696365546f6b656e0000000000000000000000000000000000000000000012')
>>> _symbol
HexBytes('0x4e49434500000000000000000000000000000000000000000000000000000008')
```

These are all 32-byte hexadecimal values, which we can convert in-line using `eth_abi.decode_single()`

```
>>> eth_abi.decode_single('uint256', _totalSupply)
291796076645200045026
>>> eth_abi.decode_single('bytes32',_name)
b'NiceToken\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x12'
>>> eth_abi.decode_single('bytes32',_symbol)
b'NICE\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08'
```

You can see how accessing the decoding the storage within the contract allows us to retrieve stored data **even when it is marked private**.

### Accessing Mappings

Accessing mapping within EVM storage is much more complicated.

If you attempted to read the storage at slot 0 (the `_balances` mapping), what happens?

```
>>> w3.eth.get_storage_at(nice.address,0)
HexBytes('0x0000000000000000000000000000000000000000000000000000000000000000')
```

Hmmm, we know something exists in that slot, but it just shows as zero.

Turns out that mappings are a special structure. Also known as a hashmap, a mapping is a one-to-one table that associates data between two groups of elements.

`_balances` is a mapping of `address` values to `uint256` values. The intent of the mapping is to keep track of the associated balance of a particular address. Accessing the `_balances` map using a ‘key’ of a certain address will return the ‘value’ associated with that address.

EVM translates all keys used by a mapping into 32-byte ‘index’ value by taking the `keccak` hash of the key plus the slot value where that mapping is stored.

Pseudocode example:

```
index = keccak(key.hex() + slot.hex())
```

To explore, let’s look up the balance for an address that we know, then attempt to decode the storage by generating the index by hand:

```
>>> nice.balanceOf(nice.address)
1083210706684838
```

The contract itself is holding a value of 1083210706684838 NICE tokens.

That value should live at the index revealed by `keccak(key.hex() + slot.hex())`.

Let’s calculate that using a combination of `web3` and `eth_utils` (a module you should already have, since Brownie depends on it):

```
>>> import eth_utils
>>> index = w3.keccak(
    hexstr=(
        eth_utils.remove_0x_prefix(
            nice.address
        ).rjust(64,'0')  
        +
        eth_utils.remove_0x_prefix(
            hex(0)
        ).rjust(64,'0')
    )
)
```

Now look up the storage value at that index, and convert it to int:

```
>>> int(w3.eth.get_storage_at(nice.address,index).hex(),16)
1083210706684838
```

The values match! Retrieving a value inside `_balances` is as “simple” as calculating the index for the associated address.

The confusing syntax of `remove_0x_prefix()` and `rjust()` is necessary to pad the hex strings to 32 bytes in length (EVM’s native word size). The keccak hash of ‘0x1’ is different from ‘0x00000001’, even though their integer conversions are the same. Be careful and always pad your hex values to 32 bytes (64 digits)!

### Mappings of Mappings

Now for one slightly more complicated, let’s take the `_allowances` mapping, which is a mapping of mappings.

Holy shit! Even though this is confusing, we can work through it and calculate the index for a mapping of mappings in a similar way.

From the NICE contract:

```
*/
function _approve(address owner, address spender, uint256 amount) internal virtual {
    require(owner != address(0), "ERC20: approve from the zero address");
    require(spender != address(0), "ERC20: approve to the zero address");

    _allowances[owner][spender] = amount;
    emit Approval(owner, spender, amount);
}
```

Notice that the allowance is set using the assignment `_allowances[owner][spender] = amount;` which tells us that the value is accessed with the form `[owner][spender]`. The approvals for the first mapping (`owner`) are stored in a sub-mapping and may have a value for multiple `spenders`.

In this case, an address holding a balance can have multiple approvals to external addresses that can call `transferFrom()`. The storage read at inner_index (`address => uint256`) allows the contract to access the `spender` approvals for a particular `owner` address, while the storage values via outer_index (`address => mapping`) provide a “pointer” to the inner mappings for those addresses.

Solidity actually does some tricky stuff with nested mappings. The maximum number of “slots” in a mapping is 2**256-1, which implies that nested mappings can also hold 2**256-1 values. This isn’t true and the EVM state can only hold 2**256-1 32-byte words in total. Solidity addresses this by “collapsing” the indexes together by taking the keccak hash of the inner mapping and keccak hashing it with the outer mapping. While introducing the technical possibilty of index collisions, the addressable space is still so large that we don’t worry about it.

The pseudo-code for the index of nested mappings is:

```
index = keccak(inner_index.hex() + keccak(outer_index.hex() + slot.hex())
```

For more info, please refer to this excellent Ethereum [Stack Exchange post discussing the storage layout of nested mappings](https://ethereum.stackexchange.com/questions/102037/storage-limit-of-2-level-mapping/102220).

Unfortunately, the NICE contract has no `getApproval()` (or similar) function to check, so we’ll just pick a [random recent approval on Etherscan](https://etherscan.io/tx/0x73d64a353da314396f07e1d113cbe642afbb2cc156088140794c9dc3d9cff149) and see if we can find it.

In this transaction, address 0xc9752cdd87bdf9470f8e077fc33a1092960689fe set an “unlimited” approval for address 0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45 (the UniswapV3 router).

Note that unlimited storage corresponds to a value of 2**256-1.

Let’s calculate the index, then fetch and decode the storage:

```
>>> allowance_index = w3.keccak(
    hexstr=(
        eth_utils.remove_0x_prefix('0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45').rjust(64, "0")
        + eth_utils.remove_0x_prefix(
            w3.keccak(
                hexstr=(
                    eth_utils.remove_0x_prefix('0xc9752cdd87bdf9470f8e077fc33a1092960689fe').rjust(64, "0")
                    + str(1).rjust(64, "0")
                )
            ).hex()
        )
    )
)
>>> allowance_index
HexBytes('0xbc7c9c93dd28a9716724406688f6645447ca96a9f08cf16e5f229e9b0248c28c')
>>> int(w3.eth.get_storage_at(nice.address, allowance_index).hex(),16)
115792089237316195423570985008687907853269984665640564039454865366202211894942
```

This number is slightly smaller than 2**256-1:

```
>>> 2**256-1 - int(w3.eth.get_storage_at(nice.address, allowance_index).hex(),16)
2718641710917744993
```

Viewing that address’s recent transactions, we see that the last time they interacted with the NICE token, it was a swap on UniswapV2. [Viewing the logs for that transaction](https://etherscan.io/tx/0x23b6cedd5112e87eaee79e00bf0ef7cf7d66aa080c75ed58f9f3ab8d5b89fc05), we see an Approval event for the amount 115792089237316195423570985008687907853269984665640564039454865366202211894942

This matches the approval retrieved from storage, so we can be reasonably confident that this is correct.

Hooray!

## Token Testing With Virtual Contracts

There’s a really neat feature of `eth_call` that allows us to override the state of an address (balance, nonce), the bytecode deployed at a particular, address, the state of all storage at that address, or just a state difference (overriding certain values but leaving others).

The goal of this section is to write a contract that will allow us to test the NICE token for a fee/tax on transfer.

We can take the runtime bytecode for this testing contract, instruct `eth_call` to pretend that it is deployed at some address, then call the testing function to check for fee/tax on transfer, _without needing to fork the blockchain or deploy the contract_.

Read that again and let it sink in.

“Devil, are you telling me that I can ‘hot-swap’ a fake contract onto an addres, then execute arbitrary function calls against a live blockchain?”

[

![](https://substackcdn.com/image/fetch/$s_!moxJ!,w_1456,c_limit,f_auto,q_auto:good,fl_progressive:steep/https%3A%2F%2Fbucketeer-e05bbc84-baa3-437e-9518-adb32be77984.s3.amazonaws.com%2Fpublic%2Fimages%2F8e796648-443a-4a05-9f31-ec26d89af91f_480x366.gif)



](https://substackcdn.com/image/fetch/$s_!moxJ!,f_auto,q_auto:good,fl_progressive:steep/https%3A%2F%2Fbucketeer-e05bbc84-baa3-437e-9518-adb32be77984.s3.amazonaws.com%2Fpublic%2Fimages%2F8e796648-443a-4a05-9f31-ec26d89af91f_480x366.gif)

Let’s write a very simple token tester contract in Vyper. It contains a single function called `test_transfer_tax` that attempts to call `transferFrom()` from `msg.sender` at the supplied address at the supplied amount.

It will compare the added balance before and after, and return True if they are not equal, otherwise False.

Any token that returns True can be assumed to be a fee/tax on transfer token, and then discarded immediately.

### Vyper Contract

`token_tester.vy`

```
# @version >=0.3

from vyper.interfaces import ERC20 as IERC20

@external
@nonpayable
def test_transfer_tax(
    token:address, 
    amount:uint256,
) -> bool:

    balance_before: uint256 = empty(uint256)
    IERC20(token).transferFrom(msg.sender, self, amount)
    balance_after: uint256 = IERC20(token).balanceOf(self)
    return balance_after - balance_before != amount
```

Get the runtime bytecode for this contract on the console:

```
(.venv) devil@hades:~/bots/contracts$ vyper -f bytecode_runtime token_tester.vy 
0x6003361161000c576100eb565b60003560e01c346100f1576331739fcd81186100e957604436186100f1576004358060a01c6100f15760405260006060526040516323b872dd6080523360a0523060c05260243560e052602060806064609c6000855af1610072573d600060003e3d6000fd5b60203d106100f1576080518060011c6100f1576101005261010050506040516370a0823160a0523060c052602060a0602460bc845afa6100b7573d600060003e3d6000fd5b60203d106100f15760a09050516080526024356080516060518082038281116100f15790509050141560a052602060a0f35b505b60006000fd5b600080fda165767970657283000306000b
```

This contract is the real end goal of the lesson, but we had to cover a lot of stuff before I could roll it out. In order for it to work, we need to learn these important prerequisites:

- Learn how to submit an `eth_call` via web3
    
- Learn how to look up the storage index for a mapping
    
- Learn how to access the storage at a contract and decode that value
    

Now that these are covered, we will add a few more steps:

- Injecting fake bytecode at an address
    
- Setting a fake balance
    
- Override storage values via a state difference dictionary
    

Luckily these can all be handled together. The [state difference dictionary is defined in the geth documentation](https://geth.ethereum.org/docs/rpc/ns-eth#eth_call), so go and read it.

For proper operation of this token tester contract, we need to set the following:

- A fake address for the token tester contract
    
- The Ether balance of our tester account (so gas checks don’t fail)
    
- The NICE token balance for our tester acount
    
- The approval amount set by the tester account to the token tester contract
    

The state different dictionary will look like this:

```
state_difference = (
    {
        tester_account_address: {"balance": fake_balance},
        tester_contract_address: {"code": FAKE_BYTECODE},
        nice.address: {
            "stateDiff": {
                allowance_index.hex(): fake_allownace,
                balance_index.hex(): fake_nice_balance,
            }
        },
    },
)
```

This accomplishes everything we need, setting a fake Ether balance for our tester account, setting a fake NICE balance (via the `balance_index` storage index), and setting a fake approval for our tester contract (via the `allowance_index` storage index).

I’ll skip a few steps here and share a tester script that will run the fee/tax transfer test on the NICE token, and display the results.

It runs on a local network against my Ethereum node, but you can run it against an RPC if you’d like.

`nice_token_tester.py`

```
from brownie import *
from eth_utils import remove_0x_prefix
import eth_abi
import web3

w3 = web3.Web3(web3.WebsocketProvider())

network.connect("mainnet-local")

FAKE_CONTRACT_ADDRESS = "0x6969696969696969696969696969696969696969"
FAKE_BYTECODE = "0x6003361161000c576100eb565b60003560e01c346100f1576331739fcd81186100e957604436186100f1576004358060a01c6100f15760405260006060526040516323b872dd6080523360a0523060c05260243560e052602060806064609c6000855af1610072573d600060003e3d6000fd5b60203d106100f1576080518060011c6100f1576101005261010050506040516370a0823160a0523060c052602060a0602460bc845afa6100b7573d600060003e3d6000fd5b60203d106100f15760a09050516080526024356080516060518082038281116100f15790509050141560a052602060a0f35b505b60006000fd5b600080fda165767970657283000306000b"

AMOUNT_TO_TEST = 1000

nice = Contract.from_explorer(
    "0x53F64bE99Da00fec224EAf9f8ce2012149D2FC88"
)

tester_account = accounts.add()

tester_contract = w3.eth.contract(
    address=FAKE_CONTRACT_ADDRESS, abi=project.load().token_tester.abi
)

# find the storage index for _balances 
# (a private mapping from address => uint256)
balance_index = w3.keccak(
    hexstr=remove_0x_prefix(
        tester_account.address
    ).rjust(64, "0")
    + str(0).rjust(64, "0")
)


# find the storage index for _allowances 
# (a private mapping from address => (mapping => uint256))
allowance_index = w3.keccak(
    hexstr=(
        remove_0x_prefix(
            tester_contract.address
        ).rjust(64, "0")
        + remove_0x_prefix(
            w3.keccak(
                hexstr=(
                    remove_0x_prefix(
                        tester_account.address
                    ).rjust(64, "0")
                    + str(1).rjust(64, "0")
                )
            ).hex()
        )
    )
)

test_taxed = eth_abi.decode_single(
    "bool",
    w3.eth.call(
        w3.eth.contract(
            address=tester_contract.address,
            abi=tester_contract.abi,
        )
        .functions.test_transfer_tax(
            nice.address, 
            AMOUNT_TO_TEST
        )
        .build_transaction(
            {
                "gas": 1_000_000,
                "from": tester_account.address,
            }
        ),
        "latest",
        {
            tester_account.address: {
                "balance": hex(10 * 10**18)
            },
            tester_contract.address: {
                "code": FAKE_BYTECODE
            },
            nice.address: {
                "stateDiff": {
                    allowance_index.hex(): (
                        "0x" + remove_0x_prefix(
                            hex(AMOUNT_TO_TEST)
                        ).rjust(64, "0")
                    ),
                    balance_index.hex(): (
                        "0x" + remove_0x_prefix(
                            hex(AMOUNT_TO_TEST)
                        ).rjust(64, "0")
                    ),
                }
            },
        },
    ),
)
print("Fee/Tax on Transfer: YES") if test_taxed else print("Fee/Tax on Transfer: NO")


state_difference = (
    {
        tester_account.address: {
            "balance": hex(FAKE_ETH_BALANCE)
        },
        tester_contract.address: {
            "code": FAKE_BYTECODE
        },
        nice.address: {
            "stateDiff": {
                allowance_index.hex(): (
                    "0x" + remove_0x_prefix(
                        hex(AMOUNT_TO_TEST)
                    ).rjust(64, "0")
                ),
                balance_index.hex(): (
                    "0x" + remove_0x_prefix(
                        hex(AMOUNT_TO_TEST)
                    ).rjust(64, "0")
                ),
            }
        },
    },
)
```

Running the script, I find the desired result:

```
(.venv) devil@hades:~/bots$ python3 nice_token_tester.py 

[...]

Fee/Tax on Transfer: YES
```

## Moving Forward

This is not an exhaustive tester, and savvy readers will notice that the storage calculation method is only valid for tokens with mappings at the storage slots that match NICE.

Not to worry, there are other methods to catch shitcoin traps that don’t rely so heavily on accessing storage values. I will be developing the technique further and will share updates.

I will write a short guide on using `eth_call` to filter for bad transactions in the next lesson, cover an improvement to the Flashbots project (multi-pool triangle arbitrage) then move on to an exploration of the UniswapV3 contracts.
