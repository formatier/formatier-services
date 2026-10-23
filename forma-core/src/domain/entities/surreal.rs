//! SurrealDB record models and value conversion helpers.
//!
//! Marker states distinguish newly created tables from persisted tables that
//! must carry a record identifier.
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

/// Represents a SurrealDB record with a type-level persistence state.
#[derive(Clone)]
pub struct Table<T, M, S> {
    id: Option<RecordId>,
    /// The table's domain data.
    pub data: T,
    /// Metadata associated with the table.
    pub metadata: M,
    _state: PhantomData<S>,
}

impl<T: SurrealValue, M: SurrealValue, S: TableState> SurrealValue for Table<T, M, S> {
    /// Returns the SurrealDB value kind represented by this table.
    fn kind_of() -> Kind {
        Record::<T, M>::kind_of()
    }

    /// Checks whether a value can be represented by this table.
    ///
    /// # Arguments
    /// * `value` - The SurrealDB value to inspect.
    ///
    /// # Returns
    /// `true` if the value matches the record representation.
    fn is_value(value: &Value) -> bool {
        Record::<T, M>::is_value(value)
    }

    /// Converts this table into its SurrealDB value representation.
    ///
    /// # Returns
    /// The encoded record value.
    fn into_value(self) -> Value {
        Record {
            id: self.id,
            data: self.data,
            metadata: self.metadata,
        }
        .into_value()
    }

    /// Decodes a SurrealDB value into a table in the requested state.
    ///
    /// # Arguments
    /// * `value` - The value to decode.
    ///
    /// # Returns
    /// The decoded table.
    ///
    /// # Errors
    /// Returns a SurrealDB error if decoding fails or a saved table has no
    /// record identifier.
    fn from_value(value: Value) -> Result<Self, SurrealError> {
        let Record { id, data, metadata } = Record::<T, M>::from_value(value)?;

        if S::HAS_ID && id.is_none() {
            return Err(SurrealError::thrown(
                "saved Table is missing its record id".to_string(),
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

impl<T: Default, M: Default> Default for Table<T, M, Created> {
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

impl<T: SurrealValue, M: SurrealValue> From<TablePairs<T, M>> for Table<T, M, Created> {
    /// Converts data and metadata into an unsaved table.
    ///
    /// # Arguments
    /// * `pairs` - Data and metadata for the new table.
    ///
    /// # Returns
    /// A table in the [`Created`] state.
    fn from(pairs: TablePairs<T, M>) -> Self {
        Self::from_pairs(pairs)
    }
}

/// Constructs unsaved tables and transitions them into the saved state.
pub trait CreatedTable<T: SurrealValue, M: SurrealValue> {
    /// Builds a table from its data and metadata.
    ///
    /// # Arguments
    /// * `pairs` - The table data and metadata.
    ///
    /// # Returns
    /// A table in the [`Created`] state.
    fn from_pairs(pairs: TablePairs<T, M>) -> Table<T, M, Created>;

    /// Assigns a record identifier and transitions the table to [`Saved`].
    ///
    /// # Arguments
    /// * `id` - The SurrealDB record identifier.
    ///
    /// # Returns
    /// The table in the saved state.
    fn with_id(self, id: RecordId) -> Table<T, M, Saved>;
}

/// Provides access to the identifier of a persisted table.
pub trait SavedTable<T, M> {
    /// Returns a reference to the table's record identifier.
    ///
    /// # Returns
    /// A reference to the persisted record identifier.
    fn id(&self) -> &RecordId;
}

impl<T: SurrealValue, M: SurrealValue> CreatedTable<T, M> for Table<T, M, Created> {
    /// Builds an unsaved table from its data and metadata.
    ///
    /// # Arguments
    /// * `pairs` - Data and metadata for the new table.
    ///
    /// # Returns
    /// A table in the [`Created`] state.
    fn from_pairs(pairs: TablePairs<T, M>) -> Table<T, M, Created> {
        Table::<T, M, Created> {
            id: None,
            data: pairs.data,
            metadata: pairs.metadata,
            _state: PhantomData,
        }
    }

    /// Assigns a record identifier and transitions the table to [`Saved`].
    ///
    /// # Arguments
    /// * `id` - SurrealDB record identifier for the persisted table.
    ///
    /// # Returns
    /// The table in the saved state.
    fn with_id(self, id: RecordId) -> Table<T, M, Saved> {
        Table::<T, M, Saved> {
            id: Some(id),
            data: self.data,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }
}

impl<T, M> SavedTable<T, M> for Table<T, M, Saved> {
    /// Returns a reference to the table's record identifier.
    ///
    /// # Returns
    /// The identifier assigned to this saved table.
    fn id(&self) -> &RecordId {
        self.id
            .as_ref()
            .expect("a saved table always has a record id")
    }
}

/// Converts a cloneable value into a SurrealDB value.
pub trait ToSurrealValue {
    /// Converts this value into its SurrealDB representation.
    ///
    /// # Returns
    /// The converted value.
    fn surreal_value(&self) -> Value;
}

impl<T> ToSurrealValue for T
where
    T: SurrealValue + Clone,
{
    /// Converts this value into its SurrealDB representation.
    ///
    /// # Returns
    /// The converted SurrealDB value.
    fn surreal_value(&self) -> Value {
        self.clone().into_value()
    }
}

/// Holds data and metadata used to construct a SurrealDB table.
#[derive(SurrealValue, Default)]
pub struct TablePairs<T: SurrealValue, M: SurrealValue> {
    #[surreal(flatten)]
    /// The table's domain data.
    pub data: T,
    #[surreal(flatten)]
    /// Metadata associated with the table.
    pub metadata: M,
}
