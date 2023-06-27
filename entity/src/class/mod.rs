pub mod email;
pub mod id;
pub mod password;
pub mod username;

macro_rules! impl_convert_string_value {
    ($ty: ty) => {
        impl TryFrom<&str> for $ty {
            type Error = crate::error::EntityError;
            fn try_from(s: &str) -> Result<Self, Self::Error> {
                <$ty as std::str::FromStr>::from_str(s)
            }
        }
        impl TryFrom<String> for $ty {
            type Error = crate::error::EntityError;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                <$ty as std::str::FromStr>::from_str(&s)
            }
        }
        impl sea_orm::TryFromU64 for $ty {
            fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
                Err(sea_orm::DbErr::Custom(format!(
                    "{} cannot be converted from u64",
                    stringify!($ty)
                )))
            }
        }

        impl From<$ty> for String {
            fn from(v: $ty) -> Self {
                v.to_string()
            }
        }
        impl From<$ty> for sea_orm::Value {
            fn from(v: $ty) -> Self {
                sea_orm::Value::String(Some(Box::new(v.to_string())))
            }
        }

        impl sea_orm::sea_query::Nullable for $ty {
            fn null() -> sea_orm::Value {
                sea_orm::Value::String(None)
            }
        }

        impl sea_orm::TryGetable for $ty {
            fn try_get_by<I: sea_orm::ColIdx>(
                res: &sea_orm::prelude::QueryResult,
                index: I,
            ) -> Result<Self, sea_orm::TryGetError> {
                let val = res.try_get_by(index).map_err(sea_orm::TryGetError::DbErr).and_then(
                    |opt: Option<_>| {
                        opt.ok_or_else(|| sea_orm::TryGetError::Null(format!("{:?}", index)))
                    },
                )?;
                Ok(<$ty as TryFrom<String>>::try_from(val)?)
            }
        }

        impl sea_orm::sea_query::ValueType for $ty {
            fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                match v {
                    sea_orm::Value::String(Some(x)) => Ok(<$ty as TryFrom<String>>::try_from(*x)?),
                    _ => Err(sea_orm::sea_query::ValueTypeErr),
                }
            }

            fn type_name() -> String {
                stringify!($ty).to_string()
            }

            fn column_type() -> sea_orm::sea_query::ColumnType {
                sea_orm::sea_query::ColumnType::String(None)
            }

            fn array_type() -> sea_orm::sea_query::ArrayType {
                sea_orm::sea_query::ArrayType::String
            }
        }
    };
}
pub(crate) use impl_convert_string_value;

macro_rules! impl_into_active_value {
    ($ty: ty) => {
        impl sea_orm::IntoActiveValue<$ty> for $ty {
            fn into_active_value(self) -> sea_orm::ActiveValue<$ty> {
                sea_orm::ActiveValue::Set(self)
            }
        }
    };
}
pub(crate) use impl_into_active_value;
