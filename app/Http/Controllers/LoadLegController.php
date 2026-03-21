<?php

namespace App\Http\Controllers;

use App\Events\LegLocationUpdated;
use App\Models\LegDocuments;
use App\Models\LegEvent;
use App\Models\LoadLegLocation;
use Illuminate\Support\Facades\Auth;
use App\Models\Equipments;
use App\Models\CommodityTypes;
use App\Models\Locations;
use App\Models\LoadType;
use App\Models\User;
use App\Models\LoadDocuments;
use App\Models\LoadHistory;
use Illuminate\Http\Request;
use App\Models\Load;
use App\Models\LoadLeg;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Storage;
use App\Models\Country;
use App\Models\CarrierPreference;
use App\Models\City;
use Illuminate\Support\Facades\Validator;
use App\Support\LoadNumbers;
use Illuminate\Validation\ValidationException;
use Illuminate\Support\Str;
use Illuminate\Validation\Rule;
use Illuminate\Support\Facades\Mail;


class LoadLegController extends Controller
{

    public function startPickup(Request $request, LoadLeg $leg)
    {
        // Validate that the leg belongs to the authenticated carrier
        $user = Auth::user();
        if ($leg->booked_carrier_id !== $user->id) {
            return redirect()->back()->with('error', 'You are not authorized to start pickup for this leg.');
        }

        // Check if the leg is in a status that allows starting pickup
        if ($leg->status_id != 8) { // Assuming 8 is the status for "Booked"
            return redirect()->back()->with('error', 'Pickup cannot be started for this leg in its current status.');
        }

        // Update the leg status to indicate pickup has started
        $leg->status_id = 5; // Assuming 9 is the status for "Pickup Started"
        $leg->pickup_started_at = now(); // Record the timestamp of pickup start
        $leg->save();

        LegEvent::create([
            'leg_id' => $leg->id,
            'type' => 'pickup_started',
        ]);
        return redirect()
            ->route('leg.track', $leg->id)
            ->with('success', 'Pickup has been started successfully. Tracking started.');

    }
    public function arrivedPickup(Request $request, LoadLeg $leg)
    {
        // Validate that the leg belongs to the authenticated carrier
        $user = Auth::user();
        if ($leg->booked_carrier_id !== $user->id) {
            return redirect()->back()->with('error', 'You are not authorized for this leg.');
        }

        // Check if the leg is in a status that allows starting pickup
        if ($leg->status_id != 5) { // Assuming 8 is the status for "Booked"
            return redirect()->back()->with('error', 'Cannot mark Arrived in current status.');
        }

        // Update the leg status to indicate pickup has started
        $leg->status_id = 6; // Assuming 9 is the status for "Pickup Started"
        $leg->pickup_arrived_at = now(); // Record the timestamp of pickup start
        $leg->save();

        LegEvent::create([
            'leg_id' => $leg->id,
            'type' => 'pickup_arrived',
        ]);
        return redirect()->back()->with('success', 'Marked as arrived at pickup.');

    }
    public function departPickup(Request $request, LoadLeg $leg)
    {
        // Validate that the leg belongs to the authenticated carrier
        $user = Auth::user();
        if ($leg->booked_carrier_id !== $user->id) {
            return redirect()->back()->with('error', 'You are not authorized for this leg.');
        }

        // Check if the leg is in a status that allows starting pickup
        if ($leg->status_id != 6) { // Assuming 8 is the status for "Booked"
            return redirect()->back()->with('error', 'Cannot depart in current status.');
        }

        // Update the leg status to indicate pickup has started
        $leg->status_id = 7; // Assuming 9 is the status for "Pickup Started"
        $leg->departed_pickup_at = now(); // Record the timestamp of pickup start
        $leg->save();

        LegEvent::create([
            'leg_id' => $leg->id,
            'type' => 'departed_pickup',
        ]);
        return redirect()->back()->with('success', 'Departed from pickup.');

    }
    public function arrivedDelivery(Request $request, LoadLeg $leg)
    {
        // Validate that the leg belongs to the authenticated carrier
        $user = Auth::user();
        if ($leg->booked_carrier_id !== $user->id) {
            return redirect()->back()->with('error', 'You are not authorized for this leg.');
        }

        // Check if the leg is in a status that allows starting pickup
        if ($leg->status_id != 7) { // Assuming 8 is the status for "Booked"
            return redirect()->back()->with('error', 'Cannot mark Arrived at Delivery in current status.');
        }

        // Update the leg status to indicate pickup has started
        $leg->status_id = 9; // Assuming 9 is the status for "Pickup Started"
        $leg->delivery_arrived_at = now(); // Record the timestamp of pickup start
        $leg->save();

        LegEvent::create([
            'leg_id' => $leg->id,
            'type' => 'delivery_arrived',
        ]);
        return redirect()->back()->with('success', 'Marked as arrived at delivery.');

    }
    public function completeDelivery(Request $request, LoadLeg $leg)
    {
        // Validate that the leg belongs to the authenticated carrier
        $user = Auth::user();
        if ($leg->booked_carrier_id !== $user->id) {
            return redirect()->back()->with('error', 'You are not authorized for this leg.');
        }

        // Check if the leg is in a status that allows starting pickup
        if ($leg->status_id != 9) { // Assuming 8 is the status for "Booked"
            return redirect()->back()->with('error', 'Cannot complete delivery in current status.');
        }

        // Update the leg status to indicate pickup has started
        $leg->status_id = 10; // Assuming 9 is the status for "Pickup Started"
        $leg->delivered_at = now(); // Record the timestamp of pickup start
        $leg->save();

        LegEvent::create([
            'leg_id' => $leg->id,
            'type' => 'delivered',
        ]);
        return redirect()->back()->with('success', 'Delivery completed, waiting for payout review.');

    }

    public function storeDocs(Request $request, LoadLeg $leg)
    {
        $user = Auth::user();

        // carrier or admin can upload
        if ($leg->booked_carrier_id !== $user->id ) { // adjust admin check
            return back()->with('error', 'You are not allowed to upload documents for this leg.');
        }

        $data = $request->validate([
            'type' => 'required|string',
            'file' => 'required|file|max:10240', // 10 MB
        ]);

        $path = $request->file('file')->store('leg-documents', 'public');

        LegDocuments::create([
            'leg_id' => $leg->id,
            'type'        => $data['type'],
            'path'        => $path,
            'meta'        => [
                'original_name' => $request->file('file')->getClientOriginalName(),
                'uploaded_by'   => $user->id,
            ],
        ]);
        return redirect()->back()->with('success', 'Document uploaded.');

    }

    public function storeLocation(Request $request, LoadLeg $leg)
    {
        $user = Auth::user();

        // Ensure this carrier owns the leg
        if ($leg->booked_carrier_id !== $user->id) {
            return response()->json(['error' => 'Not authorized'], 403);
        }

        if (!in_array($leg->status_id, [5, 6, 7, 9])) {
            return response()->json(['error' => 'Tracking not allowed in this status'], 409);
        }

        $data = $request->validate([
            'lat' => 'required|numeric|between:-90,90',
            'lng' => 'required|numeric|between:-180,180',
        ]);

        $location = LoadLegLocation::create([
            'leg_id' => $leg->id,
            'lat' => $data['lat'],
            'lng' => $data['lng'],
            'recorded_at' => now(),
        ]);
        broadcast(new LegLocationUpdated($location))->toOthers();

        return response()->json(['ok' => true]);
    }

    public function track(LoadLeg $leg)
    {
        $lastLocation = $leg->locations()->latest('recorded_at')->first();
        $events = $leg->events()->orderBy('created_at')->get();
        $documents = $leg->documents()->get();

        return view('load.track', [
            'leg' => $leg,
            'lastLocation' => $lastLocation,
            'events' => $events,
            'documents' => $documents,
        ]);
    }

}
