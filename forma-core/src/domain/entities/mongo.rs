//! MongoDB persistence models and serialization helpers.
//! Marker types distinguish new models from persisted models.
use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use mongodb::bson::{self, oid};
use serde::{Deserialize, Serialize, de::{Visitor}};
use surrealdb::types::{RecordId, SurrealValue, Value};

use crate::domain::entities::{
    Created, FormaError, FormaErrorApp, FormaErrorConverter, FormaErrorDatabase, Saved,
};

/// Represents a MongoDB document whose identifier presence is tracked by `S`.
#[derive(Serialize, Deserialize, Default)]
pub struct Model<T, M, S> {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<oid::ObjectId>,
    #[serde(flatten)]
    /// The model's domain data.
    pub data: T,
    #[serde(flatten)]
    /// Persistence metadata associated with the model.
    pub metadata: M,

    #[serde(skip)]
    _state: PhantomData<S>,
}

impl<T, M> From<ModelPairs<T, M>> for Model<T, M, Created> {
    /// Converts a pair of model data and metadata into an unsaved model.
    fn from(pairs: ModelPairs<T, M>) -> Self {
        Self::from_pairs(pairs)
    }
}

/// Constructs unsaved models and transitions them to the saved state.
pub trait CreatedModel<T, M> {
    /// Builds an unsaved model from its data and metadata.
    fn from_pairs(pairs: ModelPairs<T, M>) -> Model<T, M, Created>;

    /// Assigns an identifier and transitions the model to [`Saved`].
    fn with_id(self, id: oid::ObjectId) -> Model<T, M, Saved>;
}

/// Provides access to a persisted model's identifier.
pub trait SavedModel<T, M> {
    fn id(&self) -> oid::ObjectId;
}

impl<T, M> CreatedModel<T, M> for Model<T, M, Created> {
    /// Builds an unsaved model from its data and metadata.
    fn from_pairs(pairs: ModelPairs<T, M>) -> Model<T, M, Created> {
        Model::<T, M, Created> {
            id: None,
            data: pairs.data,
            metadata: pairs.metadata,

            _state: PhantomData,
        }
    }

    /// Assigns the database identifier and transitions the model to [`Saved`].
    fn with_id(self, id: oid::ObjectId) -> Model<T, M, Saved> {
        Model::<T, M, Saved> {
            id: Some(id),
            data: self.data,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }
}

impl<T, M> SavedModel<T, M> for Model<T, M, Saved> {
    fn id(&self) -> oid::ObjectId {
        self.id.unwrap()
    }
}

/// Converts a value into a BSON document.
pub trait ToDocument {
    /// Serializes this value as a BSON document.
    fn document(&self) -> Result<bson::Document, FormaError>;
}

impl<T> ToDocument for T
where
    T: Serialize,
{
    /// Serializes this value as a BSON document.
    fn document(&self) -> Result<bson::Document, FormaError> {
        bson::serialize_to_document(self).map_forma_err(
            FormaErrorDatabase::InvalidArgument,
            "failed to serialize model to document",
        )
    }
}

/// Holds data and metadata used to construct a MongoDB model.
#[derive(Serialize, Deserialize, Default)]
pub struct ModelPairs<T, M> {
    #[serde(flatten)]
    /// The model's domain data.
    pub data: T,
    #[serde(flatten)]
    /// Persistence metadata associated with the model.
    pub metadata: M,
}