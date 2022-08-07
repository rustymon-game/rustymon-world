use futures::StreamExt;
use log::{debug, error};
use serde::Serialize;

pub fn publish<T: Serialize + Send + Sync + 'static>(url: &str, tiles: Vec<T>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_publish(url, tiles))
}

async fn async_publish<T: Serialize + Send + Sync + 'static>(url: &str, tiles: Vec<T>) {
    let _client = reqwest::Client::new();
    let client = &_client;
    futures::stream::iter(tiles)
        .map(move |tile| async move {
            let request = client.post(url).json(&tile);
            let time = std::time::Instant::now();
            (request.send().await, time.elapsed())
        })
        .buffer_unordered(10)
        .map(|(result, duration)| {
            let error = match result {
                Ok(response) => response.error_for_status().err(),
                Err(error) => Some(error),
            };
            debug!("Request took {}ms", duration.as_millis());
            if let Some(error) = error {
                error!("Couldn't publish tile: {}", error);
            }
            ()
        })
        .all(|_| async { true })
        .await;
}
