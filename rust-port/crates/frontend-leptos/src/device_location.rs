#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export async function stloadsGetCurrentPosition() {
  if (!navigator.geolocation) {
    throw new Error('This browser does not support device geolocation.');
  }

  return await new Promise((resolve, reject) => {
    navigator.geolocation.getCurrentPosition(
      (position) => resolve({ lat: position.coords.latitude, lng: position.coords.longitude }),
      (error) => reject(new Error(error.message || 'Unable to read the current device location.')),
      {
        enableHighAccuracy: true,
        maximumAge: 10000,
        timeout: 20000,
      }
    );
  });
}

export async function stloadsStartLiveTracking(url, token) {
  if (!navigator.geolocation) {
    throw new Error('This browser does not support device geolocation.');
  }

  const headers = {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const options = {
    enableHighAccuracy: true,
    maximumAge: 10000,
    timeout: 20000,
  };

  const sendPosition = async (position) => {
    const response = await fetch(url, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        lat: position.coords.latitude,
        lng: position.coords.longitude,
      }),
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`Live tracking update failed: ${response.status} ${text}`);
    }
  };

  await new Promise((resolve, reject) => {
    navigator.geolocation.getCurrentPosition(
      async (position) => {
        try {
          await sendPosition(position);
          resolve(true);
        } catch (error) {
          reject(error);
        }
      },
      (error) => reject(new Error(error.message || 'Unable to start live tracking.')),
      options
    );
  });

  const watcherId = navigator.geolocation.watchPosition(
    (position) => {
      sendPosition(position).catch((error) => console.warn(error));
    },
    (error) => {
      console.warn(error.message || 'Live tracking permission was denied.');
    },
    options
  );

  return watcherId;
}

export function stloadsStopLiveTracking(watcherId) {
  if (!navigator.geolocation) {
    return false;
  }

  if (watcherId !== undefined && watcherId !== null) {
    navigator.geolocation.clearWatch(watcherId);
    return true;
  }

  return false;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = stloadsGetCurrentPosition)]
    async fn stloads_get_current_position() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stloadsStartLiveTracking)]
    async fn stloads_start_live_tracking(url: &str, token: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = stloadsStopLiveTracking)]
    fn stloads_stop_live_tracking(watcher_id: i32) -> bool;
}

#[cfg(target_arch = "wasm32")]
pub async fn current_position() -> Result<(f64, f64), String> {
    let value = stloads_get_current_position()
        .await
        .map_err(|error| format!("Failed to capture the current device location: {:?}", error))?;

    let lat = js_sys::Reflect::get(&value, &JsValue::from_str("lat"))
        .ok()
        .and_then(|value| value.as_f64())
        .ok_or_else(|| "Device geolocation did not return a latitude value.".to_string())?;
    let lng = js_sys::Reflect::get(&value, &JsValue::from_str("lng"))
        .ok()
        .and_then(|value| value.as_f64())
        .ok_or_else(|| "Device geolocation did not return a longitude value.".to_string())?;

    Ok((lat, lng))
}

#[cfg(target_arch = "wasm32")]
pub async fn start_live_tracking(url: &str, token: &str) -> Result<i32, String> {
    let value = stloads_start_live_tracking(url, token)
        .await
        .map_err(|error| format!("Failed to start live tracking: {:?}", error))?;

    value
        .as_f64()
        .map(|value| value as i32)
        .ok_or_else(|| "Live tracking did not return a watcher id.".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn start_live_tracking(_url: &str, _token: &str) -> Result<i32, String> {
    Err("Live tracking is only available in the browser build of the Rust UI.".into())
}

#[cfg(target_arch = "wasm32")]
pub fn stop_live_tracking(watcher_id: i32) -> bool {
    stloads_stop_live_tracking(watcher_id)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn stop_live_tracking(_watcher_id: i32) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn current_position() -> Result<(f64, f64), String> {
    Err("Device geolocation is only available in the browser build of the Rust UI.".into())
}
