use crate::qubic_types::QubicId;
use core::{fmt::Debug, str::FromStr};

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Serialize};

use crate::qubic_tcp_types::MessageType;

use super::transactions::TransactionData;

pub const QXID: QubicId = QubicId([
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
pub const TRANSFER_FEE: u64 = 1_000_000;
pub const ISSUE_ASSET_FEE: u64 = 1_000_000_000;

macro_rules! generate_packed_integers {
    ($($name: ident $alias: ty)*) => {

        $(
            #[derive(Clone, Copy, PartialEq, Eq, Hash)]
            #[repr(C)]
            pub struct $name([u8; core::mem::size_of::<$alias>()]);

            impl ToString for $name {
                fn to_string(&self) -> String {
                    <$alias>::from_le_bytes(self.0).to_string()
                }
            }

            impl Debug for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str(&self.to_string())?;

                    Ok(())
                }
            }

            impl From<$alias> for $name {
                fn from(val: $alias) -> Self {
                    $name(val.to_le_bytes())
                }
            }

            impl From<$name> for $alias {
                fn from(value: $name) -> $alias {
                    <$alias>::from_le_bytes(value.0)
                }
            }
        )*

    };
}

generate_packed_integers!(U16 u16 I16 i16 U32 u32 I32 i32 U64 u64 I64 i64);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct AssetName<const LEN: usize>(pub [u8; LEN]);

#[cfg(feature = "serde")]
pub struct AssetNameVisitor<const LEN: usize>;

#[cfg(feature = "serde")]
impl<'de, const LEN: usize> Visitor<'de> for AssetNameVisitor<LEN> {
    type Value = AssetName<LEN>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(&format!("Expected {LEN} character string"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match AssetName::<LEN>::from_str(v) {
            Ok(r) => Ok(r),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match AssetName::<LEN>::from_str(&v) {
            Ok(r) => Ok(r),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

#[cfg(feature = "serde")]
impl<const LEN: usize> Serialize for AssetName<LEN> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de, const LEN: usize> Deserialize<'de> for AssetName<LEN> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(AssetNameVisitor)
    }
}

impl<const LEN: usize> FromStr for AssetName<LEN> {
    type Err = crate::qubic_types::errors::QubicError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > LEN || !s.is_ascii() {
            return Err(
                crate::qubic_types::errors::QubicError::InvalidIdLengthError {
                    ident: "Name",
                    expected: 7,
                    found: s.len(),
                },
            );
        }

        let mut name = [0u8; LEN];

        for (idx, c) in s.as_bytes().iter().enumerate() {
            name[idx] = *c;
        }

        Ok(AssetName(name))
    }
}

impl<const LEN: usize> ToString for AssetName<LEN> {
    fn to_string(&self) -> String {
        let mut name = String::with_capacity(LEN);

        for byte in self.0 {
            if byte != 0 {
                name.push(char::from(byte));
            }
        }

        name
    }
}

impl<const LEN: usize> Debug for AssetName<LEN> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_string())?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Issuance {
    pub name: AssetName<7>,
    pub number_of_decimal_places: u8,
    pub unit_of_measurement: [u8; 7],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Ownership {
    pub padding: u8,
    pub managing_contract_index: U16,
    pub issuance_index: U32,
    pub number_of_units: I64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Possession {
    pub padding: u8,
    pub managing_contract_index: U16,
    pub issuance_index: U32,
    pub number_of_units: I64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8, C)]
pub enum AssetType {
    Empty = 0,
    Issuance(Issuance) = 1,
    Ownership(Ownership) = 2,
    Possession(Possession) = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Asset {
    pub public_key: QubicId,
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FeesInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FeesOutput {
    pub asset_issuance_fee: u32,
    pub transfer_fee: u32,
    pub trade_fee: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct TransferAssetInput {
    pub destination: QubicId,
}

impl From<TransferAssetInput> for TransactionData {
    fn from(value: TransferAssetInput) -> Self {
        Self::TransferAsset(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct IssueAssetInput {
    pub name: AssetName<8>,
    pub number_of_units: i64,
    pub unit_of_measurement: u64,
    pub number_of_decimal_places: i8,
}

impl From<IssueAssetInput> for TransactionData {
    fn from(value: IssueAssetInput) -> Self {
        Self::IssueAsset(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IssueAssetOutput {
    pub issued_number_of_units: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TransferAssetOwnershipAndPossessionInput {
    pub issuer: QubicId,
    pub possessor: QubicId,
    pub new_owner: QubicId,
    pub asset_name: AssetName<8>,
    pub number_of_units: i64,
}

set_message_type!(
    TransferAssetOwnershipAndPossessionInput,
    MessageType::BroadcastTransaction
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TranferAssetOwnershipAndPossessionOutput {
    pub transferred_number_of_units: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestIssuedAsset {
    pub public_key: QubicId,
}

set_message_type!(RequestIssuedAsset, MessageType::RequestIssuedAsset);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct RespondIssuedAsset {
    pub asset: Asset,
    pub tick: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestOwnedAsset {
    pub public_key: QubicId,
}

set_message_type!(RequestOwnedAsset, MessageType::RequestOwnedAsset);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct RespondOwnedAsset {
    pub asset: Asset,
    pub issuance_asset: Asset,
    pub tick: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestPossessedAsset {
    pub public_key: QubicId,
}

set_message_type!(RequestPossessedAsset, MessageType::RequestPossessedAsset);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct RespondPossessedAsset {
    pub asset: Asset,
    pub ownership_asset: Asset,
    pub issuance_asset: Asset,
    pub tick: u32,
}