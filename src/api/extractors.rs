use std::ops::Deref;
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::api::error::ApiResponseError;
use crate::api::server::ServerState;
use crate::store::memory::ReadingData;

pub(crate) struct ReadingDataGuard(pub(crate) Arc<ReadingData>);

impl Deref for ReadingDataGuard {
    type Target = ReadingData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<ServerState> for ReadingDataGuard {
    type Rejection = ApiResponseError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        state
            .reading_data_store
            .get()
            .map(ReadingDataGuard)
            .ok_or_else(ApiResponseError::internal_server_error)
    }
}
