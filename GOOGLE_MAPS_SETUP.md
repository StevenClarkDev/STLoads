# Google Maps API Setup Instructions

## Overview
Google Maps Places Autocomplete has been integrated into the load creation form to allow users to search and select locations easily.

## Setup Steps

### 1. Get Google Maps API Key

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the following APIs:
   - **Maps JavaScript API**
   - **Places API**
   - **Geocoding API** (optional, for additional features)

4. Create credentials:
   - Go to **APIs & Services** > **Credentials**
   - Click **Create Credentials** > **API Key**
   - Copy the generated API key

### 2. Secure Your API Key (Recommended)

1. Go to your API key settings
2. Under **Application restrictions**, select:
   - **HTTP referrers (web sites)**
3. Add your domain:
   - `https://portal.stloads.com/*`
   - `https://*.stloads.com/*`
   - For local development: `http://localhost:*`

4. Under **API restrictions**, select:
   - **Restrict key**
   - Check: Maps JavaScript API, Places API

### 3. Configure .env File

1. Open `.env` file in your project root
2. Find the line: `GOOGLE_MAPS_API_KEY=YOUR_GOOGLE_MAPS_API_KEY_HERE`
3. Replace `YOUR_GOOGLE_MAPS_API_KEY_HERE` with your actual API key:
   ```
   GOOGLE_MAPS_API_KEY=AIzaSyABC123...your_actual_key_here
   ```

### 4. Clear Configuration Cache

After updating the `.env` file, run:
```bash
php artisan config:clear
php artisan cache:clear
```

## How It Works

### User Experience

When creating a load, users can now:

1. **Type an address** in the "Pickup Location" or "Delivery Location" text field
2. **Select from autocomplete suggestions** that appear as they type
3. **Or use the dropdown** to select from previously saved locations

### Features

- **Real-time suggestions**: Google Maps provides location suggestions as the user types
- **Accurate addresses**: Selected addresses include full formatting from Google Maps
- **Country restriction**: Only US and Canada addresses are shown in autocomplete results
- **Location biasing**: Autocomplete prioritizes locations near the user's current location (based on browser geolocation)
- **Automatic location detection**: When the page loads, the browser requests permission to access the user's location to provide better suggestions

### Location Storage

- When a user selects an address from Google Maps autocomplete:
  - A new location is automatically created in the database
  - The address is saved for future reference
  - The location is linked to the user's account

- When a user selects from the dropdown:
  - The existing location from the database is used

## Troubleshooting

### Autocomplete not working

1. **Check API key**: Ensure your API key is correctly set in `.env`
2. **Check APIs enabled**: Verify that Maps JavaScript API and Places API are enabled in Google Cloud Console
3. **Check browser console**: Press F12 and look for JavaScript errors
4. **Check API restrictions**: Make sure your domain is whitelisted

### "This page can't load Google Maps correctly"

- This error usually means:
  - API key is invalid
  - Billing is not enabled in Google Cloud Console
  - API restrictions are too strict

**Solution**: Enable billing in Google Cloud Console and check API key restrictions

### Locations not saving

- Check that the `locations` table has the following columns:
  - `name` (string)
  - `address` (string)
  - `user_id` (integer)

### Geolocation permission denied

- If the browser blocks location access, the autocomplete will still work but won't prioritize nearby locations
- To enable geolocation:
  1. Click the location icon in the browser address bar
  2. Select "Allow" for location access
  3. Refresh the page
- Geolocation is optional - the autocomplete works without it, just without location biasing

### Only showing US/Canada addresses

- This is by design - the autocomplete is configured to only show addresses from the United States and Canada
- If you need other countries, update the `componentRestrictions` in `resources/views/load/add.blade.php`

## Billing Information

Google Maps Platform offers:
- **$200 free credit per month**
- After free credit, pricing applies:
  - Autocomplete (per session): $2.83 per 1000 requests
  - Places Details: $17 per 1000 requests

For most users, the free tier is sufficient.

## Support

For issues with Google Maps API:
- [Google Maps Platform Documentation](https://developers.google.com/maps/documentation)
- [Places API Documentation](https://developers.google.com/maps/documentation/places/web-service)
- [Stack Overflow - google-maps](https://stackoverflow.com/questions/tagged/google-maps)
