// use tauri::{AppHandle, Runtime};

// use crate::{Lcu, LcuExt, Result};

// #[tauri::command]
// pub(crate) async fn ping<R: Runtime>(
//     app: AppHandle<R>,
//     payload: PingRequest,
// ) -> Result<PingResponse> {
//     app.lcu().ping(payload)
// }

// impl<R: Runtime> Lcu<R> {
//     pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
//         Ok(PingResponse {
//             value: payload.value,
//         })
//     }
// }
