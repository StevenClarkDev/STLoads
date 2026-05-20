
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
