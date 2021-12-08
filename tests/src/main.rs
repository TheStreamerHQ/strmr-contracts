use serde_json::json;
use workspaces::prelude::*;

const CONTRACT_FILE: &str = "../contract/res/thestreamer_contract.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let wasm = std::fs::read(CONTRACT_FILE)?;
    let contract = worker.dev_deploy(wasm).await.unwrap();
    let outcome = worker
        .call(
            &contract.signer(),
            &contract,
            "new".to_string(),
            json!({
                "owner_id": contract.id(),
                "exchange_price_in_yocto_near": "1000000000000000000000000",
                "total_supply": "100",
                "metadata": json!({
                    "spec": "ft-1.0.0",
                    "name": "Streamer",
                    "symbol": "STRMR",
                    "decimals": 0_u8,
                }),
            })
            .to_string()
            .into_bytes(),
            None,
        )
        .await?;

    println!("initialize outcome: {:#?}", outcome);
    println!("Create a user");
    let user = worker
        .dev_create()
        .await?;

    let outcome = worker
        .call(
            &user.signer(),
            &contract,
            "buy_ft_tokens".to_string(),
            "".to_string().into_bytes(),
            Some(10_000000000000000000000000),
        ).await?;
    println!("User buys ft_tokens of TheStreamer and registers");
    println!("buy_ft_tokens outcome: {:#?}", outcome);

    let result = worker
        .view(
            contract.id().clone(),
            "ft_balance_of".to_string(),
            json!({
                "account_id": user.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?;

    println!(
        "User bought tokens and checks they balance\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    assert!(&result == "10");

    println!("User creates a subscription for all transfers where they are a receiver");

    let subscription_args = json!({
        "endpoint": "http://localhost:3030",
        "event": json!({
            "kind": "ReceiptTransferResult",
            "triggers": json!([
                json!({
                    "parameter": "ReceiverId",
                    "value": &user.id(),
                })
            ]),
        })
    });

    let outcome = worker
        .call(
            &user.signer(),
            &contract,
            "create_subscription".to_string(),
            subscription_args.to_string().into_bytes(),
            None,
        ).await?;
    println!("create_subscription outcome: {:#?}", outcome);

    let result = worker
        .view(
            contract.id().clone(),
            "subscriptions_of".to_string(),
            json!({
                "account_id": user.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?;

    println!(
        "User lists they subscriptions\n{}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    println!("Check user can delete their subscription");

    let subscription_id = &result[0]["id"];

    let outcome = worker
        .call(
            &user.signer(),
            &contract,
            "delete_subscription".to_string(),
            json!({"id": subscription_id}).to_string().into_bytes(),
            None,
        ).await?;

    println!("delete_subscription outcome: {:#?}", outcome);

    Ok(())
}
