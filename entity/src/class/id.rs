use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

// reference https://github.com/SeaQL/sea-orm/issues/402
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Uuid", into = "Uuid")]
pub struct Id<T>(Uuid, #[serde(skip_serializing)] std::marker::PhantomData<T>);

impl<T> Id<T> {
    pub fn new(id: Uuid) -> Self {
        Self(id, std::marker::PhantomData)
    }

    pub fn create() -> Self {
        Self::new(Uuid::from(ulid::Ulid::new()))
    }

    pub fn identifier(&self) -> Uuid {
        self.0
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self::new(self.identifier())
    }
}
impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl<T> std::str::FromStr for Id<T> {
    type Err = <Uuid as std::str::FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Uuid::from_str(s)?))
    }
}

impl<T> From<Uuid> for Id<T> {
    fn from(id: Uuid) -> Self {
        Self::new(id)
    }
}

impl<T> From<Id<T>> for Uuid {
    fn from(id: Id<T>) -> Self {
        id.0
    }
}

impl<T> From<Id<T>> for sea_orm::Value {
    fn from(source: Id<T>) -> Self {
        sea_orm::Value::Uuid(Some(Box::new(source.0)))
    }
}

impl<T> sea_orm::TryFromU64 for Id<T> {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Custom(format!("{} cannot be converted from u64", stringify!(Id<T>))))
    }
}

impl<T> sea_orm::TryGetable for Id<T> {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let val: Uuid = res.try_get_by(index).map_err(sea_orm::TryGetError::DbErr)?;
        Ok(Id::<T>::new(val))
    }
}

impl<T> sea_orm::sea_query::ValueType for Id<T> {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::Uuid(Some(x)) => Ok(Id::<T>::new(*x)),
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(Id).to_string()
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::Uuid
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::Uuid
    }
}
