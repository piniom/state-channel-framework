use regex::Regex;
use starknet::core::types::contract::ComputeClassHashError;
use starknet::core::types::FieldElement;
use starknet::core::types::FromStrError;
use thiserror::Error;
use url::ParseError;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("failed to parse url")]
    ParsingError(#[from] ParseError),

    #[error("Database error: {0}")]
    FromStrError(#[from] FromStrError),

    #[error("Database error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    ReadFileError(#[from] std::io::Error),
    #[error("Database error: {0}")]
    JsonError(#[from] starknet::core::types::contract::JsonError),

    #[error("Database error: {0}")]
    ClassHashError(#[from] ComputeClassHashError),

    #[error("Account error: {0}")]
    AccountError(String),
}

pub fn parse_contract_address_from_error(error_msg: &str) -> FieldElement {
    println!("Error message: {}", error_msg);
    // Define a regular expression to capture the class hash
    let re = Regex::new(
        r#"ContractAddress\(PatriciaKey\(StarkFelt\("(?P<address>0x[a-fA-F0-9]+)"\)\)\)"#,
    )
    .unwrap();

    // Attempt to capture the class hash
    if let Some(captures) = re.captures(error_msg) {
        if let Some(contract_address) = captures.name("address") {
            return FieldElement::from_hex_be(contract_address.as_str())
                .expect("Failed to parse class hash");
        }
    }

    panic!("Failed to extract class hash from error message");
}

pub fn parse_class_hash_from_error(error_msg: &str) -> FieldElement {
    println!("Error message: {}", error_msg);
    let re = Regex::new(r#"StarkFelt\("(0x[a-fA-F0-9]+)"\)"#).unwrap();

    // Attempt to capture the class hash
    if let Some(captures) = re.captures(error_msg) {
        if let Some(contract_address) = captures.get(1) {
            return FieldElement::from_hex_be(contract_address.as_str())
                .expect("Failed to parse class hash");
        }
    }

    panic!("Failed to extract class hash from error message");
}