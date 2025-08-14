<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Models\City;      // id, name, country_id
use App\Models\Country;   // id, name

class CitiesController extends Controller
{
    public function byCountry($countryId)
    {
        try {
            // validate country exists (optional but nice)
            if (!Country::whereKey($countryId)->exists()) {
                return response()->json(['message' => 'Country not found'], 404);
            }

            // keep payload light: just id + name
            $cities = City::where('country_id', $countryId)
                ->orderBy('name')
                ->limit(10000) // safety cap; adjust or remove
                ->get(['id', 'name']);

            return response()->json($cities);
        } catch (\Exception $e) {
            return response()->json(['message' => 'Failed to fetch cities'], 500);
        }
    }
}
