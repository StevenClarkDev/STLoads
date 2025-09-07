<?php

namespace App\Http\Controllers;

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


class LoadController extends Controller
{
    public function index(LogsController $logsController)
    {
        try {
            $user_id = Auth::user()->id;
            $user = User::find($user_id);
            $role     = $user->roles()->first();        // requires HasRoles
            $roleId   = $role?->id;
            if ($roleId == 1) {
                $load_legs = LoadLeg::all();
            } else if ($roleId == 3) {
                $load_legs = LoadLeg::whereNotIn('status_id', [0, 1, 7])->get();
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

    public function adminIndex(LogsController $logsController)
    {
        try {
            $load_legs = LoadLeg::all();
            $loadCount = $load_legs->count();
            $pending_load_legs = LoadLeg::where('status_id', 1)->get();
            $pendingLoadCount = $pending_load_legs->count();
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to index load in', null, null);
            return view('admin.load', compact('load_legs', 'loadCount', 'pendingLoadCount', 'pending_load_legs'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'An error occurred while processing your request.' . $e->getMessage());
        }
    }

    public function updateStatus($id, Request $request)
    {
        // Minimal validation
        $request->validate([
            'status'  => 'required|in:0,2,7',          // 1=approved, 2=rejected, 5=send back
            'remarks' => 'nullable|string|max:1000',
        ]);

        // Require remarks for reject or send back
        if (in_array((int)$request->status, [0, 7]) && !$request->filled('remarks')) {
            return back()->with('error', 'Remarks are required for Reject or Send Back.');
        }

        $load = Load::find($id);
        $user = User::find($load->user_id);
        if (!$load) {
            return back()->with('error', 'Load not found');
        }

        // Update status
        foreach ($load->legs as $leg) {
            $leg->status_id = (int)$request->status;
            $leg->save();
        }

        // Save history
        LoadHistory::create([
            'load_id'  => $load->id,
            'admin_id' => Auth::id(),
            'status'   => (int)$request->status,
            'remarks'  => $request->remarks,
        ]);

        // Email (kept simple)
        $fromAddress = config('mail.from.address');
        $fromName    = config('mail.from.name');
        $to          = $user->email;

        if ((int)$request->status === 1) {
            $subject = 'Your Load has been approved';
            $body = "Hello {$user->name},\n\nYour Load has been approved. You can now log in and start using our system.\n\nThank you,\n{$fromName}";
        } elseif ((int)$request->status === 2) {
            $subject = 'Your Load has been rejected';
            $body = "Hello {$user->name},\n\nYour Load has been rejected.\nAdmin remarks: {$request->remarks}\n\nThank you,\n{$fromName}";
        } else { // 5 = send back
            $subject = 'Action needed: Load requires revision';
            $body = "Hello {$user->name},\n\nYour Load requires revision.\nAdmin remarks: {$request->remarks}\n\nThank you,\n{$fromName}";
        }

        Mail::raw($body, function ($message) use ($to, $subject, $fromAddress, $fromName) {
            $message->from($fromAddress, $fromName)
                ->to($to)
                ->subject($subject);
        });

        return redirect()->route('admin.manage-loads')->with('success', 'Status updated.');
    }

    public function adminView(Load $load, LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is attempting to view load', null, null);
            return view('admin.load_profile', compact('load'));
        } catch (\Exception $e) {
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

    public function view(Load $load, LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is attempting to view load', null, null);
            return view('load.load_profile', compact('load'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->with('error', 'An error occurred while processing your request.' . $e->getMessage());
        }
    }

    public function save(Load $load, Request $request)
    {
        // Basic validation
        $request->validate([
            'doc_id'      => 'array',
            'doc_id.*'    => [
                'nullable',
                'integer',
                Rule::exists('load_documents', 'id')->where(fn($q) => $q->where('load_id', $load->id)),
            ],
            'doc_name'    => 'required|array|min:1',
            'doc_name.*'  => 'required|string|max:255',
            'doc_type'    => 'required|array',
            'doc_type.*'  => ['required', Rule::in(['standard', 'blockchain'])],
            'documents'   => 'array',
            'documents.*' => 'nullable|file|mimes:jpeg,jpg,png,pdf,docx'
                . '|mimetypes:image/jpeg,image/png,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document'
                . '|max:20480',
        ]);

        $ids   = $request->input('doc_id', []);
        $names = $request->input('doc_name', []);
        $types = $request->input('doc_type', []);
        $files = $request->file('documents', []);

        // Require file for NEW rows (where doc_id is empty)
        foreach ($names as $i => $name) {
            if (empty($ids[$i]) && empty($files[$i])) {
                throw ValidationException::withMessages([
                    "documents.$i" => "File is required for new document row #" . ($i + 1) . ".",
                ]);
            }
        }

        DB::transaction(function () use ($load, $ids, $names, $types, $files) {
            $keepIds = [];

            foreach ($names as $i => $name) {
                $docId = $ids[$i] ?? null;
                $type  = strtolower($types[$i] ?? 'standard');
                $file  = $files[$i] ?? null;

                if ($docId) {
                    // Update existing
                    $doc = LoadDocuments::where('load_id', $load->id)->where('id', $docId)->firstOrFail();
                } else {
                    // Create new
                    $doc = new LoadDocuments(['load_id' => $load->id]);
                }

                // If a new file is uploaded, replace file + metadata
                if ($file) {
                    if ($doc->exists && $doc->file_path) {
                        Storage::disk('public')->delete($doc->file_path);
                    }
                    $path = $file->store('loads/documents', 'public');

                    $doc->file_path     = $path;
                    $doc->original_name = $file->getClientOriginalName();
                    $doc->mime_type     = $file->getClientMimeType();
                    $doc->file_size     = $file->getSize();
                }

                // Always update name/type
                $doc->document_name = trim($name);
                $doc->document_type = $type;

                // Blockchain handling
                if ($type === 'blockchain') {
                    // Need a file_path (either existing or just uploaded)
                    if (!$doc->file_path) {
                        throw ValidationException::withMessages([
                            "documents.$i" => "A file is required to anchor blockchain document row #" . ($i + 1) . ".",
                        ]);
                    }
                    $abs = Storage::disk('public')->path($doc->file_path);
                    $hash = hash_file('sha256', $abs);

                    $doc->hash           = $hash;
                    $doc->hash_algorithm = 'sha256';

                    // Create a new mock tx when there is a new upload or no tx yet
                    if ($file || empty($doc->mock_blockchain_tx)) {
                        $doc->mock_blockchain_tx        = (string) Str::uuid();
                        $doc->mock_blockchain_timestamp = now();
                    }
                } else {
                    // Clear blockchain fields when switching to standard
                    $doc->hash = null;
                    $doc->hash_algorithm = null;
                    $doc->mock_blockchain_tx = null;
                    $doc->mock_blockchain_timestamp = null;
                }

                $doc->save();
                $keepIds[] = $doc->id;
            }

            // Delete documents removed in the UI (i.e., not submitted)
            LoadDocuments::where('load_id', $load->id)
                ->whereNotIn('id', $keepIds)
                ->delete();

            // Set status back to "Awaiting admin approval"
            foreach ($load->legs as $leg) {
                $leg->status_id = 1;
                $leg->save();
            }
        });

        return back()->with('success', 'Laod updated. We’ll review your changes shortly.');
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

            'doc_name'        => 'required|array|min:1',
            'doc_name.*'      => 'required|string|max:255',
            'doc_type'        => 'required|array',
            'doc_type.*'      => ['required', Rule::in(['standard', 'blockchain'])],
            'documents'       => 'required|array|min:1',
            'documents.*'     => 'required|file|mimes:jpeg,jpg,png,pdf,docx'
                . '|mimetypes:image/jpeg,image/png,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document'
                . '|max:20480',
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

        if (
            count($request->input('doc_name', [])) !== count($request->input('doc_type', [])) ||
            count($request->input('doc_name', [])) !== count($request->file('documents', []))
        ) {
            throw ValidationException::withMessages([
                'documents' => 'Document name, type, and file count must match.',
            ]);
        }

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

            // if ($request->hasFile('documents')) {
            //     $path = $request->file('documents')->store('loads/documents', 'public');
            //     $load->document_path = $path;
            // }

            $load->save();

            $names = $request->input('doc_name', []);
            $types = $request->input('doc_type', []);
            $files = $request->file('documents', []);

            foreach ($names as $i => $name) {
                $file = $files[$i];
                $storedPath = $file->store('loads/documents', 'public');

                // $storedPath = $file->store("load_documents/{$load->id}", 'public');

                $documentType = strtolower($types[$i] ?? 'standard');

                $payload = [
                    'load_id'       => $load->id,
                    'document_name' => trim($name),
                    'document_type' => $documentType,
                    'file_path'     => $storedPath,
                    'original_name' => $file->getClientOriginalName(),
                    'mime_type'     => $file->getClientMimeType(),
                    'file_size'     => $file->getSize(),
                ];

                // Mock blockchain anchoring for blockchain docs
                if ($documentType === 'blockchain') {
                    $absPath = Storage::disk('public')->path($storedPath);
                    $hash = hash_file('sha256', $absPath);

                    $payload['hash']                     = $hash;
                    $payload['hash_algorithm']           = 'sha256';
                    $payload['mock_blockchain_tx']       = (string) Str::uuid(); // fake tx id
                    $payload['mock_blockchain_timestamp'] = now();
                }

                LoadDocuments::create($payload);
            }

            $user_id = Auth::user()->id;
            $user = User::find($user_id);
            $role     = $user->roles()->first();        // requires HasRoles
            $roleId   = $role?->id;
            $loadNumber = LoadNumbers::generateLoadNumber($roleId);

            // Update load with number
            $load->load_number = $loadNumber;
            $load->save();

            $rowCount = count($request->pickup_location);
            $legNo = 1;

            for ($i = 0; $i < $rowCount; $i++) {
                $loadLeg = new LoadLeg();
                $loadLeg->load_id               = $load->id;
                $loadLeg->leg_no                = $legNo; // 1..n
                $loadLeg->leg_code              = LoadNumbers::legCode($loadNumber, $legNo);

                $loadLeg->pickup_location_id    = $request->pickup_location[$i];
                $loadLeg->delivery_location_id  = $request->delivery_location[$i];
                $loadLeg->pickup_date           = $request->pickup_date[$i];
                $loadLeg->delivery_date         = $request->delivery_date[$i];
                $loadLeg->bid_status            = $request->bid_status[$i];
                $loadLeg->price                 = $request->price[$i];
                $loadLeg->status_id             = 1;
                $loadLeg->save();
                $legNo++;
            }

            $load->leg_count = $rowCount;
            $load->save();

            DB::commit();

            $logsController->createLog(__METHOD__, 'success', 'User has successfully added a load', $load, null);

            return redirect()->route('manage-loads')->with('success', 'Load Created Successfully');
        } catch (\Throwable $e) {
            DB::rollBack();
            $logsController->createLog(__METHOD__, 'error', 'Failed to create load: ' . $e->getMessage(), null, null);
            return back()->withInput()->with('error', 'An error occurred while processing your request. ' . $e->getMessage());
        }
    }

    public function book(LoadLeg $load_leg, Request $request, LogsController $logsController)
    {
        // (Optional) ensure only allowed users can book
        // $this->authorize('book', $load_leg);

        // Accept an amount, but we'll prefer server-side price below
        $validated = $request->validate([
            'amount' => ['nullable', 'numeric', 'min:0'],
        ]);

        try {
            return DB::transaction(function () use ($load_leg, $validated, $logsController) {

                $leg = \App\Models\LoadLeg::query()
                    ->whereKey($load_leg->getKey())
                    ->lockForUpdate()
                    ->firstOrFail();

                if ((int) $leg->status_id === 4 || $leg->booked_at || $leg->booked_carrier_id) {
                    $logsController->createLog(__METHOD__, 'warning', 'Attempted to book an already booked load', $leg, null);
                    return redirect()
                        ->route('manage-loads')
                        ->with('error', 'This load has already been booked.');
                }

                $amount = $leg->price ?? ($validated['amount'] ?? 0);

                $leg->update([
                    'status_id'         => 4,                 // e.g., 4 = booked
                    'booked_carrier_id' => Auth::id(),
                    'booked_at'         => now(),
                    'booked_amount'     => $amount,
                ]);

                $logsController->createLog(__METHOD__, 'success', 'User has successfully booked a load', $leg, null);

                return redirect()
                    ->route('manage-loads')
                    ->with('success', 'Load booked successfully.');
            });
        } catch (\Throwable $e) {
            try {
                $logsController->createLog(__METHOD__, 'error', 'Failed to book load: ' . $e->getMessage(), $load_leg, null);
            } catch (\Throwable $ignored) {
            }

            return back()
                ->withInput()
                ->with('error', 'An error occurred while processing your request.');
        }
    }
}
