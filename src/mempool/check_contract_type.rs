use crate::primitive::ContractType;
use serde_json::Value;

pub fn check_account_type(code: &Value) -> ContractType {
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
        // Handle JSON object as a SpecialCaseContract
        Value::Object(_) => ContractType::SpecialCaseContract,
        // Assume any other type is an EOA
        _ => ContractType::ExternallyOwnedAccount,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_contract_account() {
        assert_eq!(
            check_account_type(&Value::Null),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            check_account_type(&Value::String("0x".to_string())),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            check_account_type(&Value::String("0x0".to_string())),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            check_account_type(&Value::String("".to_string())),
            ContractType::ExternallyOwnedAccount
        );
        assert_eq!(
            check_account_type(&Value::String("0x123".to_string())),
            ContractType::ContractAccount
        );
        assert_eq!(
            check_account_type(&Value::String("{...}".to_string())),
            ContractType::SpecialCaseContract
        );
        assert_eq!(
            check_account_type(&Value::Object(serde_json::Map::new())),
            ContractType::SpecialCaseContract
        );
    }
}
