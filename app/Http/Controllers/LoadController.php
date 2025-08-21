<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\Auth;
use App\Models\Equipments;
use App\Models\CommodityTypes;
use App\Models\Locations;
use App\Models\LoadType;
use App\Models\User;
use Illuminate\Http\Request;
use App\Models\Load;
use App\Models\LoadLeg;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Storage;
use App\Models\Country;
use App\Models\CarrierPreference;
use App\Models\City;
use Illuminate\Support\Facades\Validator;

class LoadController extends Controller
{
    public function index(LogsController $logsController)
    {
        try {
            $user_id = Auth::user()->id;
            $user = User::find($user_id);
            $role     = $user->roles()->first();        // requires HasRoles
            $roleId   = $role?->id;
            if ($roleId == 1 || $roleId == 3) {
                $load_legs = LoadLeg::all();
            } else {
                // $load_legs = LoadLeg::with('load_master')->where('load_master.user_id', $user_id)->get();
                $load_legs = LoadLeg::with('load_master')
                    ->whereHas('load_master', function ($query) use ($user_id) {
                        $query->where('user_id', $user_id);  // Make sure user_id exists in load_master
                    })
                    ->get();
            }
            $loadCount = $load_legs->count();
            $load_types = LoadType::all();
            $equipments = Equipments::all();
            $commodity_types = CommodityTypes::all();
            $locations = Locations::all();
            $countries = Country::orderBy('name')->get(['id', 'name']);

            $carrierPreference = $user->carrierPreference;
            $cities = null;
            $recommendedLoadLegs = null;


            if ($carrierPreference) {
                $carrierPreference->equipment_id = json_decode($carrierPreference->equipment_id);
                $carrierPreference->load_type_id = json_decode($carrierPreference->load_type_id);
                $carrierPreference->country_id = json_decode($carrierPreference->country_id);
                $carrierPreference->city_id = json_decode($carrierPreference->city_id);
                $carrierPreference->availability_days = json_decode($carrierPreference->availability_days);

                $cities = City::whereIn('country_id', $carrierPreference->country_id)
                    ->orderBy('name')
                    ->get(['id', 'name']);

                $recommendedLoadLegs = $load_legs->map(function ($load_leg) use ($carrierPreference) {
                    $score = 0;
                    $debug = []; // To store debug info for what is being matched

                    // Match Country and City (pickupLocation or deliveryLocation)
                    $pickupLocationMatch = in_array($load_leg->pickupLocation->country_id, $carrierPreference->country_id) &&
                        in_array($load_leg->pickupLocation->city_id, $carrierPreference->city_id);
                    $deliveryLocationMatch = in_array($load_leg->deliveryLocation->country_id, $carrierPreference->country_id) &&
                        in_array($load_leg->deliveryLocation->city_id, $carrierPreference->city_id);

                    if ($pickupLocationMatch || $deliveryLocationMatch) {
                        $score++;
                        $debug[] = 'Location matched: ' . ($pickupLocationMatch ? 'Pickup ' : 'Delivery ') .
                            'Country: ' . $load_leg->pickupLocation->country->name .
                            ', City: ' . $load_leg->pickupLocation->city->name;
                    }

                    // Match Equipment and Load Type
                    $equipmentMatch = in_array($load_leg->load_master->equipment_id, (array) $carrierPreference->equipment_id);
                    $loadTypeMatch = in_array($load_leg->load_master->load_type_id, (array) $carrierPreference->load_type_id);
                    if ($equipmentMatch) {
                        $score++;
                        $debug[] = 'Equipment matched: ' . $load_leg->load_master->equipment->name;
                    }
                    if ($loadTypeMatch) {
                        $score++;
                        $debug[] = 'Load Type matched: ' . $load_leg->load_master->load_type->name;
                    }

                    // Match Weight
                    $weightMatch = $load_leg->load_master->weight <= $carrierPreference->max_weight_capacity;
                    if ($weightMatch) {
                        $score++;
                        $debug[] = 'Weight matched: ' . $load_leg->load_master->weight . ' <= ' . $carrierPreference->max_weight_capacity;
                    }

                    // Match Availability Days (pickup or delivery)
                    $pickupDayMatch = $this->isAvailableOnDay($load_leg->pickup_date, $carrierPreference->availability_days);
                    $deliveryDayMatch = $this->isAvailableOnDay($load_leg->delivery_date, $carrierPreference->availability_days);
                    if ($pickupDayMatch || $deliveryDayMatch) {
                        $score++;
                        $debug[] = 'Availability Day matched: ' . ($pickupDayMatch ? 'Pickup ' : 'Delivery ') .
                            'Day: ' . ($pickupDayMatch ? $load_leg->pickup_date : $load_leg->delivery_date);
                    }

                    // Add the score to the load leg object
                    $load_leg->score = $score;
                    $load_leg->debug_info = $debug; // Store the debug information in the load leg object

                    return $load_leg;
                });

                // Sort the load legs by their score (highest score first)
                $recommendedLoadLegs = $recommendedLoadLegs->sortByDesc('score');
            }

            $recommendedLoadLegsCount = $recommendedLoadLegs ? $recommendedLoadLegs->count() : 0;

            $logsController->createLog(__METHOD__, 'success', 'User is attempting to index load in', null, null);
            // dd('here');
            return view('load.index', compact('load_legs', 'roleId', 'loadCount', 'equipments', 'load_types', 'commodity_types', 'locations', 'countries', 'cities', 'carrierPreference', 'recommendedLoadLegs', 'recommendedLoadLegsCount', 'user'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'An error occurred while processing your request.' . $e->getMessage());
        }
    }

    // Helper function to match availability days (pickup or delivery)
    private function isAvailableOnDay($date, $availabilityDays)
    {
        // Convert date to the day of the week (e.g., 'Monday', 'Tuesday', etc.)
        $dayOfWeek = \Carbon\Carbon::parse($date)->format('l'); // 'Monday', 'Tuesday', etc.

        // Check if the availability days include the current day
        return in_array(strtolower($dayOfWeek), array_map('strtolower', $availabilityDays));
    }

    public function savePreferences(Request $request, LogsController $logsController)
    {
        try {
            // Validate the incoming request data
            $validated = $request->validate([
                'equipment_id' => 'required|array',
                'load_type_id' => 'required|array',
                'country_id' => 'required|array',
                'city_id' => 'required|array',
                'availability_days' => 'required|array',
                'max_weight_capacity' => 'required|numeric',
            ]);

            // Get the authenticated user's ID
            $user_id = Auth::user()->id;

            // Save or update the user's preferences in the CarrierPreference model
            CarrierPreference::updateOrCreate(
                ['user_id' => $user_id],
                [
                    'equipment_id' => json_encode($request->equipment_id),
                    'load_type_id' => json_encode($request->load_type_id),
                    'country_id' => json_encode($request->country_id),
                    'city_id' => json_encode($request->city_id),
                    'availability_days' => json_encode($request->availability_days),
                    'max_weight_capacity' => $request->max_weight_capacity,
                ]
            );

            $logsController->createLog(__METHOD__, 'success', 'User is attempting to savePreferences', null, null);

            // Return success response
            return response()->json(['success' => true]);
        } catch (\Exception $e) {
            // Log the exception for debugging (optional)
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            // Return an error response
            return response()->json(['success' => false, 'message' => 'There was an error saving your preferences. Please try again.' . $e->getMessage()]);
        }
    }

    public function add(LogsController $logsController)
    {
        try {
            $load_types = LoadType::all();
            $equipments = Equipments::all();
            $commodity_types = CommodityTypes::all();
            $locations = Locations::all();

            $user_id = Auth::user()->id;
            $user = User::find($user_id);
            $role     = $user->roles()->first();        // requires HasRoles
            $roleId   = $role?->id;

            $logsController->createLog(__METHOD__, 'success', 'User is attempting to add load in', null, null);
            return view('load.add', compact('load_types', 'equipments', 'commodity_types', 'locations', 'roleId', 'user_id'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'An error occurred while processing your request.' . $e->getMessage());
        }
    }

    public function store(Request $request, LogsController $logsController)
    {
        $validator = Validator::make($request->all(), [
            'title'               => ['required', 'string', 'max:255'],
            'load_type_id'        => ['required', 'integer', 'exists:load_types,id'],
            'equipment_id'        => ['required', 'integer', 'exists:equipments,id'],
            'commodity_type_id'   => ['required', 'integer', 'exists:commodity_types,id'],
            'weight_unit'         => ['required', 'in:LBS,KG,MTON'],
            'weight'              => ['required', 'numeric', 'min:0'],

            // file: 10 MB + allowed types (adjust as you like)
            'documents'           => ['nullable', 'file', 'max:10240', 'mimes:pdf,jpg,jpeg,png,webp,doc,docx'],
            'special_instructions' => ['nullable', 'string', 'max:2000'],

            'is_hazardous'             => ['nullable'],
            'is_temperature_controlled' => ['nullable'],

            'pickup_location'     => ['required', 'array', 'min:1'],
            'pickup_location.*'   => ['required', 'integer', 'exists:locations,id'],

            'delivery_location'   => ['required', 'array', 'min:1'],
            'delivery_location.*' => ['required', 'integer', 'exists:locations,id'],

            'pickup_date'         => ['required', 'array', 'min:1'],
            'pickup_date.*'       => ['required', 'date'],

            'delivery_date'       => ['required', 'array', 'min:1'],
            'delivery_date.*'     => ['required', 'date'], // <-- remove after_or_equal here

            'bid_status'          => ['required', 'array', 'min:1'],
            'bid_status.*'        => ['required', 'in:Fixed,Open'],

            'price'               => ['required', 'array', 'min:1'],
            'price.*'             => ['required', 'numeric', 'min:0'],
        ]);

        // per-index date check
        $validator->after(function ($v) use ($request) {
            $rowCount = count($request->pickup_date ?? []);
            for ($i = 0; $i < $rowCount; $i++) {
                $p = $request->pickup_date[$i] ?? null;
                $d = $request->delivery_date[$i] ?? null;
                if ($p && $d && strtotime($d) < strtotime($p)) {
                    $v->errors()->add("delivery_date.$i", 'Delivery date must be on/after the pickup date.');
                }
            }
        });

        $validated = $validator->validate();

        // Make sure all leg arrays have the same length
        $counts = [
            count($request->pickup_location ?? []),
            count($request->delivery_location ?? []),
            count($request->pickup_date ?? []),
            count($request->delivery_date ?? []),
            count($request->bid_status ?? []),
            count($request->price ?? []),
        ];
        if (count(array_unique($counts)) !== 1) {
            return back()->withInput()->withErrors([
                'load_legs' => 'Each load leg row must have all fields filled.',
            ]);
        }

        try {
            DB::beginTransaction();

            $load = new Load();
            $load->title = $request->title;
            $load->load_type_id = $request->load_type_id;
            $load->equipment_id = $request->equipment_id;
            $load->commodity_type_id = $request->commodity_type_id;
            $load->weight_unit = $request->weight_unit;
            $load->weight = $request->weight;
            $load->special_instructions = $request->input('special_instructions');
            $load->is_hazardous = $request->boolean('is_hazardous');
            $load->is_temperature_controlled = $request->boolean('is_temperature_controlled');
            $load->user_id = Auth::id();
            $load->status = 1;

            if ($request->hasFile('documents')) {
                $path = $request->file('documents')->store('loads/documents', 'public');
                $load->document_path = $path;
            }

            $load->save();

            $rowCount = count($request->pickup_location);
            for ($i = 0; $i < $rowCount; $i++) {
                $loadLeg = new LoadLeg();
                $loadLeg->load_id               = $load->id;
                $loadLeg->pickup_location_id    = $request->pickup_location[$i];
                $loadLeg->delivery_location_id  = $request->delivery_location[$i];
                $loadLeg->pickup_date           = $request->pickup_date[$i];
                $loadLeg->delivery_date         = $request->delivery_date[$i];
                $loadLeg->bid_status            = $request->bid_status[$i];
                $loadLeg->price                 = $request->price[$i];
                $loadLeg->save();
            }

            DB::commit();

            $logsController->createLog(__METHOD__, 'success', 'User has successfully added a load', $load, null);

            return redirect()->route('manage-loads')->with('success', 'Load Created Successfully');
        } catch (\Throwable $e) {
            DB::rollBack();
            $logsController->createLog(__METHOD__, 'error', 'Failed to create load: ' . $e->getMessage(), null, null);
            return back()->withInput()->with('error', 'An error occurred while processing your request. ' . $e->getMessage());
        }
    }
}
