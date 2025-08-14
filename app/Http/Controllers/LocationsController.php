<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Http\Controllers\Controller;
use App\Models\Locations;
use App\Models\Country;
use App\Models\City;
use Illuminate\View\View;
use Illuminate\Http\RedirectResponse;

class LocationsController extends Controller
{
    public function index(LogsController $logsController): View|RedirectResponse
    {
        try {
            $locations = Locations::all();
            $logsController->createLog(__METHOD__, 'success', 'User is viewing Locations list', null, null);
            return view('locations.index', compact('locations'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to list Locations: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to load data: ' . $e->getMessage());
        }
    }

    public function create(LogsController $logsController): View|RedirectResponse
    {
        try {
            $countries = Country::orderBy('name')->get(['id','name']);
            $logsController->createLog(__METHOD__, 'success', 'User is opening Location create form', null, null);
            return view('locations.create', compact('countries'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open Location create form: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open create form: ' . $e->getMessage());
        }
    }

    public function store(Request $request, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            Locations::create([
                'name' => $request->name,
                'city_id' => $request->city_id,
                'country_id' => $request->country_id,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User created a new Location', null, null);
            return redirect()->route('locations.index')->with('success', 'Location created successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create Location: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to create Location: ' . $e->getMessage());
        }
    }

    public function show($id, LogsController $logsController): View|RedirectResponse
    {
        dd('This method is not implemented yet.'); // Placeholder for future implementation
    }

    public function edit($id, LogsController $logsController): View|RedirectResponse
    {
        try {
            $location = Locations::findOrFail($id);
            $countries = Country::orderBy('name')->get(['id','name']);
            $cities = City::where('country_id', $location->country_id)
                      ->orderBy('name')
                      ->get(['id','name']);
            $logsController->createLog(__METHOD__, 'success', 'User opened edit form for Location (ID: ' . $id . ')', null, null);
            return view('locations.edit', compact('location', 'countries', 'cities'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to open edit form for Location (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to open edit form: ' . $e->getMessage());
        }
    }

    public function update(Request $request, $id, LogsController $logsController): RedirectResponse
    {
        try {
            $request->validate([
                'name' => 'required'
            ]);

            $location = Locations::findOrFail($id);
            $location->update([
                'name' => $request->name,
                'city_id' => $request->city_id,
                'country_id' => $request->country_id,
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User updated Location (ID: ' . $id . ')', null, null);
            return redirect()->route('locations.index')->with('success', 'Location updated successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to update Location (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to update Location: ' . $e->getMessage());
        }
    }

    public function destroy($id, LogsController $logsController): RedirectResponse
    {
        try {
            $location = Locations::findOrFail($id);
            $location->delete();

            $logsController->createLog(__METHOD__, 'success', 'User deleted Location (ID: ' . $id . ')', null, null);
            return redirect()->route('locations.index')->with('success', 'Location deleted successfully');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to delete Location (ID: ' . $id . '): ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'Failed to delete Location: ' . $e->getMessage());
        }
    }
}
