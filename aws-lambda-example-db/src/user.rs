use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

/// Incoming payload for user creation/upsert requests.
#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "familyId")]
    pub family_id: String,
}

/// Representation of a user record persisted in DynamoDB.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRecord {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub email: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "familyId")]
    pub family_id: String,
}

impl UserRecord {
    /// Build a new record from a payload, assigning ids/timestamps as needed.
    pub fn new(payload: CreateUserPayload) -> Self {
        let now = Utc::now();
        Self {
            user_id: payload
                .user_id
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_name: payload.user_name,
            email: payload.email,
            created_at: now,
            updated_at: now,
            family_id: payload.family_id,
        }
    }

    /// Convert the record into a DynamoDB attribute map.
    pub fn into_item(self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert("userId".into(), AttributeValue::S(self.user_id));
        map.insert("userName".into(), AttributeValue::S(self.user_name));
        map.insert("email".into(), AttributeValue::S(self.email));
        map.insert(
            "createdAt".into(),
            AttributeValue::S(self.created_at.to_rfc3339()),
        );
        map.insert(
            "updatedAt".into(),
            AttributeValue::S(self.updated_at.to_rfc3339()),
        );
        map.insert("familyId".into(), AttributeValue::S(self.family_id));
        map
    }

    /// Rehydrate a record from a DynamoDB attribute map.
    pub fn from_item(item: HashMap<String, AttributeValue>) -> Result<Self, AppError> {
        let get_str = |key: &str| -> Result<String, AppError> {
            item.get(key)
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::Dynamo(format!("missing attribute `{key}`")))
        };
        let created_at = get_str("createdAt")?
            .parse::<DateTime<Utc>>()
            .map_err(|_| AppError::Dynamo("invalid createdAt timestamp".into()))?;
        let updated_at = get_str("updatedAt")?
            .parse::<DateTime<Utc>>()
            .map_err(|_| AppError::Dynamo("invalid updatedAt timestamp".into()))?;
        Ok(Self {
            user_id: get_str("userId")?,
            user_name: get_str("userName")?,
            email: get_str("email")?,
            created_at,
            updated_at,
            family_id: get_str("familyId")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_round_trip() {
        let payload = CreateUserPayload {
            user_id: Some("user-123".into()),
            user_name: "tester".into(),
            email: "user@example.com".into(),
            password: "pw".into(),
            family_id: "family-1".into(),
        };
        let record = UserRecord::new(payload);
        assert_eq!(record.user_id, "user-123");
        let item = record.clone().into_item();
        let rehydrated = UserRecord::from_item(item).expect("roundtrip");
        assert_eq!(rehydrated.user_name, "tester");
        assert_eq!(rehydrated.email, "user@example.com");
        assert_eq!(rehydrated.family_id, "family-1");
    }
}
