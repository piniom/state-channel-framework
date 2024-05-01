use super::account::MockAccount;
use super::models::{Quote, RequestQuotation, RequestQuotationResponse};
use super::price::get_btc_usdt_price;
use crate::request::account::scalar_to_hex;
use crate::request::models::Nonce;
use crate::server::ServerError;
use axum::response::IntoResponse;
use axum::Json;
use rand::rngs::OsRng;

pub async fn request_quote(
    Json(payload): Json<RequestQuotation>,
) -> Result<impl IntoResponse, ServerError> {
    let nonce = Nonce::new(32);
    let btc_price = match get_btc_usdt_price().await {
        Ok(price) => price,
        Err(err) => {
            eprintln!("Error getting BTC price: {:?}", err);
            return Err(ServerError::BTCRequestFailure(
                "Failed to get BTC price.".to_string(),
            ));
        }
    };
    let quote = Quote {
        address: payload.address,
        quantity: payload.quantity,
        nonce: nonce.clone(),
        price: btc_price,
    };

    let mut rng = OsRng;
    let mock_account = MockAccount::new(&mut rng);
    let quote_json = serde_json::to_string(&quote).unwrap();
    let server_signature = mock_account.sign_message(&quote_json.as_bytes(), &mut rng);
    let server_signature = match server_signature {
        Ok(signature) => {
            let signature_json = format!(
                "{{\"r\": \"{}\", \"s\": \"{}\"}}",
                scalar_to_hex(&signature.r),
                scalar_to_hex(&signature.s)
            );
            println!("Serialized Signature: {}", signature_json);
            signature_json
        }
        Err(e) => {
            //todo: fix the error
            println!("Failed to sign message: {}", e);
            return Err(ServerError::DatabaseError("ERROR".to_string()));
        }
    };

    Ok(Json(RequestQuotationResponse {
        quote,
        server_signature: server_signature,
    }))
}