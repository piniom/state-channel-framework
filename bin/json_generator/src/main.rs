use crate::requests::{create_agreement, request_settlement_proof_with_price_and_data};
use axum::Router;
use clap::Parser;
use generate_data::generate_identical_but_shuffled_prices;
use rand_core::OsRng;
use serde::ser::StdError;
use server::request::account::MockAccount;
use server::request::models::AppState;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use to_json::{prepare_and_save_data, save_out};
mod generate_data;
pub mod models;
pub mod requests;
mod to_json;

const URL_ACCEPT_CONTRACT: &str = "/acceptContract";
const URL_REQUEST_QUOTE: &str = "/requestQuoteWithPrice";
const URL_REQUEST_SETTLEMENT_PROOF_WITH_DATA: &str = "/requestSettlementProofWithPriceAndData";
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from("http://localhost:7005/server/requestQuote"))]
    url_request_quote: String,

    #[arg(long, default_value_t = String::from("http://localhost:7005/server/acceptContract"))]
    url_accept_contract: String,

    #[arg(long, default_value_t = String::from("http://localhost:7005/server/requestSettlementProof"))]
    url_request_settlement_proof: String,

    #[arg(short, long, default_value_t = 1)]
    agreements_count: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let args: Args = Args::parse();
    let agreements_count = args.agreements_count / 2;
    let (buy_prices, sell_prices) = generate_identical_but_shuffled_prices(agreements_count);

    let address = "test_case";
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to initialize the database");
    let _ = db.use_ns("test").use_db("test").await;
    let mut rng = OsRng;
    let server_mock_account = MockAccount::new(&mut rng);
    let state: AppState = AppState {
        db,
        mock_account: server_mock_account.clone(),
    };

    let mut rng = OsRng;
    let client_mock_account = MockAccount::new(&mut rng);

    let router: Router = server::request::router(&state);

    //first 50 buys then 50 sells with the same sum prices
    for buying_price in buy_prices {
        create_agreement(
            1,
            buying_price as i64,
            address,
            URL_REQUEST_QUOTE,
            URL_ACCEPT_CONTRACT,
            router.clone(),
            client_mock_account.clone(),
        )
        .await?;
    }

    for selling_price in sell_prices {
        create_agreement(
            -1,
            selling_price as i64,
            address,
            URL_REQUEST_QUOTE,
            URL_ACCEPT_CONTRACT,
            router.clone(),
            client_mock_account.clone(),
        )
        .await?;
    }

    let settlement_price = 1500i64;
    // Request settlement
    let settlement_proof = request_settlement_proof_with_price_and_data(
        URL_REQUEST_SETTLEMENT_PROOF_WITH_DATA,
        &address.to_string(),
        settlement_price,
        router.clone(),
    )
    .await?;
    //Save to files
    // TODO : HARDCODED FILE PATH
    let path_in: &str = "bin/json_generator/output/in.json";
    prepare_and_save_data(
        path_in.to_string(),
        settlement_proof.clone(),
        client_mock_account.clone(),
        server_mock_account.clone(),
    )
    .await?;
    let path_out: &str = "bin/json_generator/output/out.json";
    save_out(
        path_out.to_string(),
        settlement_price,
        settlement_proof.diff,
    )
    .await?;
    Ok(())
}
