use crate::apply::apply_agreements;
use crate::declare::declare_contract;
use crate::deploy::deploy_contract;
use crate::errors::RunnerError;
use crate::models::get_agreements_data;
use clap::Parser;
use starknet::core::types::FieldElement;
use url::Url;
mod apply;
mod declare;
mod deploy;
mod errors;
mod get_account;
mod models;
use get_account::get_account;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short, env)]
    rpc_url: Url,

    #[arg(long, short, env)]
    chain_id: FieldElement,

    #[arg(long, short, env)]
    address: FieldElement,

    #[arg(long, short, env)]
    private_key: FieldElement,

    #[arg(long, short, env)]
    udc_address: FieldElement,

    #[arg(long, short, env)]
    salt: FieldElement,
}

#[tokio::main]
async fn main() -> Result<(), RunnerError> {
    let args: Args = Args::parse();
    let (agreements, client_public_key, server_public_key) = get_agreements_data()?;

    let prefunded_account = get_account(
        args.rpc_url.clone(),
        args.chain_id,
        args.address,
        args.private_key,
    );

    let class_hash: FieldElement = declare_contract(&prefunded_account).await?;

    let deployment_address = deploy_contract(
        prefunded_account,
        client_public_key,
        server_public_key,
        class_hash,
        args.salt,
        args.udc_address,
    )
    .await?;

    let now = Instant::now();
    {
        apply_agreements(
            agreements,
            deployment_address.deployed_address,
            args.rpc_url,
            args.chain_id,
            args.address,
            args.private_key,
        )
        .await;
    }
    println!("Elapsed: {:.2?}", now.elapsed());

    Ok(())
}