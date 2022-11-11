near-amm
==================

This app was initialized with [create-near-app]


Quick Start
===========

If you haven't installed dependencies during setup:

    npm install


Build and deploy your contract to TestNet with a temporary dev account:

    npm run deploy

Test your contract:

    npm test


Exploring The Code
==================

1. The smart-contract code lives in the `/contract` folder. See the README there for
   more info. In blockchain apps the smart contract is the "backend" of your app.
2. Test your contract: `npm test`, this will run the tests in `integration-tests` directory.


Test in near testnet via `near-cli`
======

Frist, login my account as `zhangtao1596.testnet`

    near login

Create sub account to deploy A fungible token

    near create-account a.zhangtao1596.testnet --masterAccount zhangtao1596.testnet

Deploy A token with initial function call

    near deploy --accountId a.zhangtao1596.testnet --wasmFile ./integration-tests/res/fungible_token.wasm --initFunction new --initArgs '{"owner_id":"zhangtao1596.testnet","total_supply":"100000000000","metadata":{"spec":"ft-1.0.0","name":"token a","symbol":"a","decimals":8}}'

View owner balance

    near view a.zhangtao1596.testnet ft_balance_of '{"account_id":"zhangtao1596.testnet"}'

Deploy token B in the same way

    near create-account b.zhangtao1596.testnet --masterAccount zhangtao1596.testnet --initialBalance 50

    near deploy --accountId b.zhangtao1596.testnet --wasmFile ./integration-tests/res/fungible_token.wasm --initFunction new --initArgs '{"owner_id":"zhangtao1596.testnet","total_supply":"100000000000","metadata":{"spec":"ft-1.0.0","name":"token b","symbol":"b","decimals":8}}'

    near view b.zhangtao1596.testnet ft_balance_of '{"account_id":"zhangtao1596.testnet"}'

Deploy amm contract with initial function call

    near create-account amm.zhangtao1596.testnet --masterAccount zhangtao1596.testnet --initialBalance 10

    near deploy --accountId amm.zhangtao1596.testnet --wasmFile  contract/target/wasm32-unknown-unknown/release/near_amm.wasm --initFunction init --initArgs '{"owner_id":"zhangtao1596.testnet","ft_a_id":"a.zhangtao1596.testnet","ft_b_id":"b.zhangtao1596.testnet"}'

Now we can view tokens info and initial ratio

    near view amm.zhangtao1596.testnet get_tokens_info
    near view amm.zhangtao1596.testnet get_ratio

Owner add A token to amm

    near call a.zhangtao1596.testnet storage_deposit '{"account_id":"amm.zhangtao1596.testnet","registration_only":true}' --deposit 0.008 --accountId zhangtao1596.testnet

    near call a.zhangtao1596.testnet ft_transfer_call '{"receiver_id":"amm.zhangtao1596.testnet","amount":"20000000000","msg":"0"}'  --accountId zhangtao1596.testnet --depositYocto 1 --gas 200000000000000

Now we can get A token balance of amm contract

    near view a.zhangtao1596.testnet ft_balance_of '{"account_id":"amm.zhangtao1596.testnet"}'

Add B token to amm contract in the same way as add A token

    near call b.zhangtao1596.testnet storage_deposit '{"account_id":"amm.zhangtao1596.testnet","registration_only":true}' --deposit 0.008 --accountId zhangtao1596.testnet

    near call b.zhangtao1596.testnet ft_transfer_call '{"receiver_id":"amm.zhangtao1596.testnet","amount":"20000000000","msg":"0"}'  --accountId zhangtao1596.testnet --depositYocto 1 --gas 200000000000000

    near view b.zhangtao1596.testnet ft_balance_of '{"account_id":"amm.zhangtao1596.testnet"}'

Now we can see the ratio updated

    near view amm.zhangtao1596.testnet get_ratio

Now create a new sub account `Alice` to swap A token to B token

    near create-account alice.zhangtao1596.testnet --masterAccount zhangtao1596.testnet --initialBalance 10

Regist alice in A token and B token

    near call a.zhangtao1596.testnet storage_deposit '{"account_id":"alice.zhangtao1596.testnet","registration_only":true}' --deposit 0.008 --accountId zhangtao1596.testnet

    near call b.zhangtao1596.testnet storage_deposit '{"account_id":"alice.zhangtao1596.testnet","registration_only":true}' --deposit 0.008 --accountId zhangtao1596.testnet

Transfer initial A balance

    near call a.zhangtao1596.testnet ft_transfer '{"receiver_id":"alice.zhangtao1596.testnet","amount":"5000000000"}'  --accountId zhangtao1596.testnet --depositYocto 1

Now we can see alice has `5000000000` A token and `0` B token

    near view a.zhangtao1596.testnet ft_balance_of '{"account_id":"alice.zhangtao1596.testnet"}'
    near view b.zhangtao1596.testnet ft_balance_of '{"account_id":"alice.zhangtao1596.testnet"}'

Alice send A to amm to swap B

    near call a.zhangtao1596.testnet ft_transfer_call '{"receiver_id":"amm.zhangtao1596.testnet","amount":"5000000000","msg":"0"}'  --accountId alice.zhangtao1596.testnet --depositYocto 1 --gas 200000000000000

Now we can see alice has `0` A and `4000000000` B

    near view a.zhangtao1596.testnet ft_balance_of '{"account_id":"alice.zhangtao1596.testnet"}'
    near view b.zhangtao1596.testnet ft_balance_of '{"account_id":"alice.zhangtao1596.testnet"}'

# Build a simple AMM contract #

The contract should include the following methods:

Initialization method:
• Input is the address of the contract owner and the addresses of two tokens (hereinafter token A and token B).
• The method requests and stores the metadata of tokens (name, decimals)
• Creates wallets for tokens А & В.
The method for getting information about the contract (ticker, decimals, ratio of tokens A and B)
 
Deposit method:
• The user can transfer a certain number of tokens A to the contract account and in return must receive a certain number of tokens B (similarly in the other direction). The contract supports a certain ratio of tokens A and B. X * Y = K (K is some constant value, X and Y are the number of tokens A and B respectively.
• The owner of the contract can transfer a certain amount of tokens A or B to the contract account, thereby changing the ratio K.
Implementation requirements in order of their priority.
• Implement contact. The contract must work with two tokens with an arbitrary number of decimals.
• Smart contact should be tested.
• Instructions in the readme: contract building, deployment, creation of a token, contract initialization, contract testing description.
Materials:

• https://nomicon.io 

• https://docs.near.org/docs/develop/basics/getting-started 
 
Please note that you will have 5 days to perform the task, so the due date will Nov 1(by the end of the day). In case you have any questions, feel free to let me know. Good luck!
