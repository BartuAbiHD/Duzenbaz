use crate::error::{AppError, AppResult};

/// Ağır dosya işlerini arayüzü dondurmadan ayrı iş parçacığında çalıştırır.
pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> AppResult<T> + Send + 'static) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|_| AppError::msg("İşlem beklenmedik şekilde durdu. Lütfen tekrar deneyin."))?
}
