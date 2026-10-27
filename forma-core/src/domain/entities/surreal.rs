//! SurrealDB record models and value conversion helpers.
//! SurrealDB table models and conversion helpers.
use std::marker::PhantomData;
use surrealdb::types::{Error as SurrealError, Kind, RecordId, SurrealValue, Value};

use crate::domain::entities::{Created, Saved, TableState};

/// SurrealDB value representation for a table and its flattened fields.
#[derive(SurrealValue)]
struct Record<T: SurrealValue, M: SurrealValue> {
    id: Option<RecordId>,
    #[surreal(flatten)]
    data: T,
    #[surreal(flatten)]
    metadata: M,
}

pub enum UpdaterMode<K: 'static> {
    Include(&'static [K]),
    Exclude(&'static [K]),
}

pub trait UpdaterFilter<K: PartialEq + 'static> {
    const FILTER: UpdaterMode<K>;

    fn filter_iter<J, V, I>(iter: I) -> impl Iterator<Item = (J, V)>
    where
        J: PartialEq<K>,
        I: Iterator<Item = (J, V)>,
    {
        let (keys, if_key_listed) = match Self::FILTER {
            UpdaterMode::Include(keys) => (keys, true),
            UpdaterMode::Exclude(keys) => (keys, false),
        };

        iter.filter(move |(j, _)| keys.iter().any(|k| j == k) == if_key_listed)
    }
}

/// Represents a SurrealDB record with a type-level persistence state.
#[derive(Clone)]
pub struct Table<T, M, S>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
    S: TableState,
{
    id: Option<RecordId>,
    /// The table's domain data.
    pub data: T,
    /// Metadata associated with the table.
    pub metadata: M,
    _state: PhantomData<S>,
}

impl<T, M, S> SurrealValue for Table<T, M, S>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
    S: TableState,
{
    fn kind_of() -> Kind {
        Record::<T, M>::kind_of()
    }

    /// Checks whether a value can be represented by this table.
    fn is_value(value: &Value) -> bool {
        Record::<T, M>::is_value(value)
    }

    /// Converts this table into its SurrealDB value representation.
    fn into_value(self) -> Value {
        Record {
            id: self.id,
            data: self.data,
            metadata: self.metadata,
        }
        .into_value()
    }

    /// Decodes a SurrealDB value into a table in the requested state.
    fn from_value(value: Value) -> Result<Self, SurrealError> {
        let Record { id, data, metadata } = Record::<T, M>::from_value(value)?;

        if S::HAS_ID && id.is_none() {
            return Err(SurrealError::thrown(
                "saved table is missing its record ID".to_string(),
            ));
        }

        Ok(Self {
            id,
            data,
            metadata,
            _state: PhantomData,
        })
    }
}

impl<T: Default, M: Default> Default for Table<T, M, Created>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
{
    /// Creates an unsaved table with default data and metadata.
    fn default() -> Self {
        Self {
            id: None,
            data: T::default(),
            metadata: M::default(),
            _state: PhantomData,
        }
    }
}

impl<T, M> From<TablePairs<T, M>> for Table<T, M, Created>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
{
    /// Converts data and metadata into an unsaved table.
    fn from(pairs: TablePairs<T, M>) -> Self {
        Self::from_pairs(pairs)
    }
}

/// Constructs unsaved tables and transitions them into the saved state.
pub trait CreatedTable<T, M>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
{
    /// Builds a table from its data and metadata.
    fn from_pairs(pairs: TablePairs<T, M>) -> Table<T, M, Created>;

    /// Assigns a record identifier and transitions the table to [`Saved`].
    fn with_id(self, id: RecordId) -> Table<T, M, Saved>;
}

/// Provides access to the identifier of a persisted table.
pub trait SavedTable<T, M> {
    fn id(&self) -> &RecordId;
}

impl<T, M> CreatedTable<T, M> for Table<T, M, Created>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
{
    /// Builds an unsaved table from its data and metadata.
    fn from_pairs(pairs: TablePairs<T, M>) -> Table<T, M, Created> {
        Table::<T, M, Created> {
            id: None,
            data: pairs.data,
            metadata: pairs.metadata,
            _state: PhantomData,
        }
    }

    /// Assigns a record identifier and transitions the table to [`Saved`].
    fn with_id(self, id: RecordId) -> Table<T, M, Saved> {
        Table::<T, M, Saved> {
            id: Some(id),
            data: self.data,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }
}

impl<T, M> SavedTable<T, M> for Table<T, M, Saved>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
{
    fn id(&self) -> &RecordId {
        self.id
            .as_ref()
            .expect("a saved table always has a record id")
    }
}

/// Converts a cloneable value into a SurrealDB value.
pub trait ToSurrealValue {
    /// Converts this value into its SurrealDB representation.
    fn surreal_value(&self) -> Value;
}

impl<T> ToSurrealValue for T
where
    T: SurrealValue + Clone,
{
    /// Converts this value into its SurrealDB representation.
    fn surreal_value(&self) -> Value {
        self.clone().into_value()
    }
}

/// Holds data and metadata used to construct a SurrealDB table.
#[derive(SurrealValue, Default, Clone)]
pub struct TablePairs<T, M>
where
    T: SurrealValue + Clone,
    M: SurrealValue + Clone,
{
    #[surreal(flatten)]
    /// The table's domain data.
    pub data: T,
    #[surreal(flatten)]
    /// Metadata associated with the table.
    pub metadata: M,
}
