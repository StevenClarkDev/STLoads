#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export async function stloadsLoadGooglePlaces(apiKey) {
  if (!apiKey || !apiKey.trim()) {
    throw new Error('GOOGLE_MAPS_API_KEY is not configured for the frontend build.');
  }

  if (window.google && window.google.maps && window.google.maps.places) {
    return true;
  }

  if (window.__stloadsGooglePlacesLoader) {
    return window.__stloadsGooglePlacesLoader;
  }

  window.__stloadsGooglePlacesLoader = new Promise((resolve, reject) => {
    const existing = document.querySelector('script[data-stloads-google-places="true"]');
    if (existing) {
      existing.addEventListener('load', () => resolve(true), { once: true });
      existing.addEventListener('error', () => reject(new Error('Google Maps script failed to load.')), { once: true });
      return;
    }

    const script = document.createElement('script');
    script.src = `https://maps.googleapis.com/maps/api/js?key=${encodeURIComponent(apiKey)}&libraries=places&loading=async`;
    script.async = true;
    script.defer = true;
    script.dataset.stloadsGooglePlaces = 'true';
    script.onload = () => resolve(true);
    script.onerror = () => reject(new Error('Google Maps script failed to load.'));
    document.head.appendChild(script);
  });

  return window.__stloadsGooglePlacesLoader;
}

function setInputValueAndDispatch(id, value) {
  const input = document.getElementById(id);
  if (!input) return;
  input.value = value || '';
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.dispatchEvent(new Event('change', { bubbles: true }));
}

export async function stloadsAttachAddressAutocomplete(inputId, cityId, countryId, placeIdId, latitudeId, longitudeId) {
  const input = document.getElementById(inputId);
  if (!input || input.dataset.googleAutocompleteAttached === 'true') {
    return true;
  }

  if (!(window.google && window.google.maps && window.google.maps.places)) {
    throw new Error('Google Maps Places library is not loaded.');
  }

  const options = {
    types: ['geocode', 'establishment'],
    fields: ['formatted_address', 'geometry', 'name', 'address_components', 'place_id'],
    componentRestrictions: { country: ['us', 'ca'] },
  };

  const autocomplete = new google.maps.places.Autocomplete(input, options);
  autocomplete.setOptions({ strictBounds: false });

  if (navigator.geolocation) {
    navigator.geolocation.getCurrentPosition(
      (position) => {
        const { latitude, longitude } = position.coords;
        const biasDelta = 4.5;
        autocomplete.setBounds({
          north: latitude + biasDelta,
          south: latitude - biasDelta,
          east: longitude + biasDelta,
          west: longitude - biasDelta,
        });
      },
      () => {
        // No GPS or permission denied. Leave predictions unbiased so Google returns general suggestions.
      },
      { enableHighAccuracy: false, maximumAge: 300000, timeout: 2500 }
    );
  }

  input.addEventListener('input', () => input.classList.add('loading'));

  autocomplete.addListener('place_changed', () => {
    input.classList.remove('loading');
    const place = autocomplete.getPlace();
    if (!place || !place.formatted_address) {
      return;
    }

    let city = '';
    let country = '';
    if (Array.isArray(place.address_components)) {
      for (const component of place.address_components) {
        if (component.types && component.types.includes('locality')) {
          city = component.long_name;
        } else if (component.types && component.types.includes('administrative_area_level_1') && !city) {
          city = component.long_name;
        }
        if (component.types && component.types.includes('country')) {
          country = component.long_name;
        }
      }
    }

    const latitude = place.geometry && place.geometry.location ? place.geometry.location.lat() : '';
    const longitude = place.geometry && place.geometry.location ? place.geometry.location.lng() : '';

    setInputValueAndDispatch(inputId, place.formatted_address || place.name || '');
    setInputValueAndDispatch(cityId, city);
    setInputValueAndDispatch(countryId, country);
    setInputValueAndDispatch(placeIdId, place.place_id || '');
    setInputValueAndDispatch(latitudeId, latitude === '' ? '' : String(latitude));
    setInputValueAndDispatch(longitudeId, longitude === '' ? '' : String(longitude));
  });

  input.dataset.googleAutocompleteAttached = 'true';
  return true;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = stloadsLoadGooglePlaces)]
    async fn stloads_load_google_places(api_key: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = stloadsAttachAddressAutocomplete)]
    async fn stloads_attach_address_autocomplete(
        input_id: &str,
        city_id: &str,
        country_id: &str,
        place_id_id: &str,
        latitude_id: &str,
        longitude_id: &str,
    ) -> Result<JsValue, JsValue>;
}

#[cfg(target_arch = "wasm32")]
pub async fn ensure_loaded(api_key: &str) -> Result<(), String> {
    stloads_load_google_places(api_key)
        .await
        .map(|_| ())
        .map_err(|error| format!("Google Places load failed: {:?}", error))
}

#[cfg(target_arch = "wasm32")]
pub async fn attach_address_autocomplete(
    input_id: &str,
    city_id: &str,
    country_id: &str,
    place_id_id: &str,
    latitude_id: &str,
    longitude_id: &str,
) -> Result<(), String> {
    stloads_attach_address_autocomplete(
        input_id,
        city_id,
        country_id,
        place_id_id,
        latitude_id,
        longitude_id,
    )
    .await
    .map(|_| ())
    .map_err(|error| format!("Google Places attach failed: {:?}", error))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn ensure_loaded(_api_key: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn attach_address_autocomplete(
    _input_id: &str,
    _city_id: &str,
    _country_id: &str,
    _place_id_id: &str,
    _latitude_id: &str,
    _longitude_id: &str,
) -> Result<(), String> {
    Ok(())
}
