use anyhow::Ok;
use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::json;
use std::{env, fs};
use workspaces::{types::Gas, Account, Contract};

const FUNGIBLE_TOKEN_PATH: &str = "./res/fungible_token.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wasm_arg: &str = &(env::args().nth(1).unwrap());
    let wasm_filepath = fs::canonicalize(env::current_dir()?.join(wasm_arg))?;

    let worker = workspaces::sandbox().await?;

    let amm_wasm = std::fs::read(wasm_filepath)?;
    let ft_wasm = std::fs::read(FUNGIBLE_TOKEN_PATH)?;

    let ft_a = worker.dev_deploy(&ft_wasm).await?;
    let ft_b = worker.dev_deploy(&ft_wasm).await?;
    let amm = worker.dev_deploy(&amm_wasm).await?;

    let account = worker.dev_create_account().await?;
    let owner = account
        .create_subaccount("owner")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let alice = account
        .create_subaccount("alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    ft_a.call("new")
        .args_json(json!({
            "owner_id": owner.id(),
            "total_supply": "100000000000",
            "metadata": json!({
                 "spec": "ft-1.0.0",
                 "name": "token a",
                 "symbol": "a",
                 "decimals": 8,
            })
        }))
        .transact()
        .await?
        .into_result()?;
    ft_b.call("new")
        .args_json(json!({
            "owner_id": owner.id(),
            "total_supply": "100000000000",
            "metadata": json!({
                 "spec": "ft-1.0.0",
                 "name": "token b",
                 "symbol": "b",
                 "decimals": 8,
            })
        }))
        .transact()
        .await?
        .into_result()?;
    // begin tests
    test_init(&owner, &amm, &ft_a, &ft_b).await?;
    test_get_tokens_info(&alice, &amm).await?;
    test_get_ratio(&owner, &amm).await?;
    test_deposit_by_owner(&owner, &amm, &ft_a, &ft_b).await?;
    test_exchange_a_to_b(&owner, &alice, &ft_a, &ft_b, &amm).await?;
    Ok(())
}

async fn test_init(
    owner: &Account,
    amm: &Contract,
    ft_a: &Contract,
    ft_b: &Contract,
) -> anyhow::Result<()> {
    println!("test amm init...");
    owner
        .call(amm.id(), "init")
        .args_json(json!({
            "owner_id": owner.id(),
            "ft_a_id": ft_a.id(),
            "ft_b_id": ft_b.id(),
        }))
        .gas(parse_gas!("100 Tgas") as Gas)
        .transact()
        .await?
        .into_result()?;
    println!("      Passed ✅ init success");
    Ok(())
}

async fn test_get_tokens_info(user: &Account, contract: &Contract) -> anyhow::Result<()> {
    println!("test get tokens info...");
    let message: String = user
        .call(contract.id(), "get_tokens_info")
        .args_json(json!({}))
        .transact()
        .await?
        .json()?;
    println!("{}", &message);
    println!("      Passed ✅ get tokens info success");
    Ok(())
}

async fn test_get_ratio(user: &Account, amm: &Contract) -> anyhow::Result<()> {
    println!("test get ratio...");
    let message: u128 = user
        .call(amm.id(), "get_ratio")
        .args_json(json!({}))
        .transact()
        .await?
        .json()?;
    println!("{}", &message);
    println!("      Passed ✅ get ratio success");
    Ok(())
}

async fn test_deposit_by_owner(
    owner: &Account,
    amm: &Contract,
    ft_a: &Contract,
    ft_b: &Contract,
) -> anyhow::Result<()> {
    println!("test deposit by owner...");
    owner
        .call(ft_a.id(), "storage_deposit")
        .args_json(json!({
            "account_id": amm.id(),
            "registration_only": true,
        }))
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?
        .into_result()?;
    owner
        .call(ft_b.id(), "storage_deposit")
        .args_json(json!({
            "account_id": amm.id(),
        }))
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?
        .into_result()?;
    println!("owner add a token");
    let mut result = owner
        .call(ft_a.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": amm.id(),
            "amount": "20000000000",
            "msg": "0",
        }))
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as Gas)
        .transact()
        .await?;
    println!("{:?}", result.logs());
    println!("owner add b token");
    result = owner
        .call(ft_b.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": amm.id(),
            "amount": "20000000000",
            "msg": "0",
        }))
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as Gas)
        .transact()
        .await?;
    println!("{:?}", result.logs());
    let ratio: u128 = owner
        .call(amm.id(), "get_ratio")
        .args_json(json!({}))
        .transact()
        .await?
        .json()?;
    assert_eq!(ratio, 400000000000000000000);
    println!("      Passed ✅ add liquid by owner success");
    Ok(())
}

async fn test_exchange_a_to_b(
    owner: &Account,
    alice: &Account,
    ft_a: &Contract,
    ft_b: &Contract,
    amm: &Contract,
) -> anyhow::Result<()> {
    println!("test alice exchange a to b...");
    owner
        .call(ft_a.id(), "storage_deposit")
        .args_json(json!({
            "account_id": alice.id(),
            "registration_only": true,
        }))
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?
        .into_result()?;
    owner
        .call(ft_b.id(), "storage_deposit")
        .args_json(json!({
            "account_id": alice.id(),
            "registration_only": true,
        }))
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?
        .into_result()?;
    owner
        .call(ft_a.id(), "ft_transfer")
        .args_json(json!({
            "receiver_id": alice.id(),
            "amount": "5000000000"
        }))
        .deposit(1)
        .transact()
        .await?
        .into_result()?;
    let result = alice
        .call(ft_a.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": amm.id(),
                "amount": "5000000000",
                "msg": "0",
        }))
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as Gas)
        .transact()
        .await?
        .into_result()?;
    println!("{:?}", result.logs());
    let balance: U128 = owner
        .call(ft_b.id(), "ft_balance_of")
        .args_json(json!({
            "account_id": alice.id(),
        }))
        .transact()
        .await?
        .json()?;
    assert_eq!(u128::from(balance), 4000000000);
    println!("      Passed ✅ exchange a to b success");
    Ok(())
}
