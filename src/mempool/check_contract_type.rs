use crate::primitive::ContractType;
use serde_json::Value;

pub fn is_contract_account(code: &Value) -> ContractType {
    match code {
        Value::Null => ContractType::ExternallyOwnedAccount,
        Value::String(s) => {
            let trim_stringed_code = s.trim();

            if trim_stringed_code.starts_with("{") || trim_stringed_code.starts_with("[") {
                ContractType::SpecialCaseContract
            } else {
                let code_without_prefix = trim_stringed_code.trim_start_matches("0x");
                if code_without_prefix.is_empty() || code_without_prefix == "0" {
                    ContractType::ExternallyOwnedAccount
                } else {
                    ContractType::ContractAccount
                }
            }
        }
        Value::Object(_) => ContractType::SpecialCaseContract, // Handle JSON object as a SpecialCaseContract
        _ => ContractType::ExternallyOwnedAccount,             // Assume any other type is an EOA
    }
}

impl ContractType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContractType::ExternallyOwnedAccount => "ExternallyOwnedAccount",
            ContractType::ContractAccount => "ContractAccount",
            ContractType::SpecialCaseContract => "SpecialCaseContract",
        }
    }
}
