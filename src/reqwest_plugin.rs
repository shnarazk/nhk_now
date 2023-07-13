//! This file was copied from https://github.com/TotalKrill/bevy_mod_reqwest
/// and modified for dev-0.11.0.
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
pub use reqwest;

use {bevy::tasks::Task, futures_lite::future};

#[derive(Resource)]
pub struct ReqwestClient(pub reqwest::Client);
impl Default for ReqwestClient {
    fn default() -> Self {
        Self(reqwest::Client::new())
    }
}

/// we have to use an option to be able to ".take()" later
#[derive(Component, Deref)]
pub struct ReqwestRequest(pub Option<reqwest::Request>);

impl From<reqwest::Request> for ReqwestRequest {
    fn from(val: reqwest::Request) -> Self {
        ReqwestRequest(Some(val))
    }
}

#[derive(Component, Deref)]
struct ReqwestInflight(pub Task<reqwest::Result<bytes::Bytes>>);

#[derive(Component, Deref)]
pub struct ReqwestBytesResult(pub reqwest::Result<bytes::Bytes>);

impl ReqwestBytesResult {
    pub fn as_str(&self) -> Option<&str> {
        match &self.0 {
            Ok(string) => Some(std::str::from_utf8(string).ok()?),
            Err(_) => None,
        }
    }
    pub fn as_string(&mut self) -> Option<String> {
        Some(self.as_str()?.into())
    }
    pub fn deserialize_json<'de, T: serde::Deserialize<'de>>(&'de mut self) -> Option<T> {
        serde_json::from_str(self.as_str()?).ok()
    }
}

pub struct ReqwestPlugin;
impl Plugin for ReqwestPlugin {
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<ReqwestClient>() {
            app.init_resource::<ReqwestClient>();
        }
        app.add_systems(Update, Self::start_handling_requests);
        app.add_systems(Update, Self::poll_inflight_requests_to_bytes);
    }
}

//TODO: Make type generic, and we can create systems for JSON and TEXT requests
impl ReqwestPlugin {
    fn start_handling_requests(
        mut commands: Commands,
        http_client: ResMut<ReqwestClient>,
        mut requests: Query<(Entity, &mut ReqwestRequest), Added<ReqwestRequest>>,
    ) {
        let thread_pool = AsyncComputeTaskPool::get();
        for (entity, mut request) in requests.iter_mut() {
            bevy::log::debug!("Creating: {entity:?}");
            // if we take the data, we can use it
            if let Some(request) = request.0.take() {
                let client = http_client.0.clone();

                let task = {
                    thread_pool.spawn(async move {
                        async_compat::Compat::new(async {
                            client.execute(request).await?.bytes().await
                        })
                        .await
                    })
                };
                // put it as a component to be polled, and remove the request, it has been handled
                commands.entity(entity).insert(ReqwestInflight(task));
                commands.entity(entity).remove::<ReqwestRequest>();
            }
        }
    }

    fn poll_inflight_requests_to_bytes(
        mut commands: Commands,
        // Very important to have the Without, otherwise we get task failure upon completed task
        mut requests: Query<(Entity, &mut ReqwestInflight), Without<ReqwestBytesResult>>,
    ) {
        for (entity, mut request) in requests.iter_mut() {
            bevy::log::debug!("polling: {entity:?}");

            if let Some(result) = future::block_on(future::poll_once(&mut request.0)) {
                // move the result over to a new component
                commands
                    .entity(entity)
                    .insert(ReqwestBytesResult(result))
                    .remove::<ReqwestInflight>();
            }
        }
    }
}
