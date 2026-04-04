<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\City;
use App\Models\Country;
use App\Models\Equipments;
use App\Models\Load;
use App\Models\LoadLeg;
use App\Models\LoadType;
use App\Models\Locations;
use App\Models\StloadsHandoff;
use App\Support\LoadNumbers;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Validator;

class TmsInboundController extends Controller
{
    /**
     * POST /api/stloads/push
     *
     * Receive a board-ready load from TMS, create Load + Legs immediately,
     * and mark the handoff as published.
     */
    public function push(Request $request): JsonResponse
    {
        $validator = $this->validatePayload($request);

        if ($validator->fails()) {
            return response()->json([
                'status'  => 'validation_error',
                'errors'  => $validator->errors(),
            ], 422);
        }

        try {
            DB::beginTransaction();

            $handoff = $this->createHandoff($request, StloadsHandoff::STATUS_PUSH_IN_PROGRESS);
            $handoff->recordEvent('push_started', $request->input('pushed_by'), $request->input('source_module'));

            $load = $this->materializeLoad($request, $handoff);

            $handoff->markPublished($load->id);
            $handoff->recordEvent('published', $request->input('pushed_by'), $request->input('source_module'), 'success');

            DB::commit();

            return response()->json([
                'status'      => 'published',
                'handoff_id'  => $handoff->id,
                'load_id'     => $load->id,
                'load_number' => $load->load_number,
            ], 201);
        } catch (\Throwable $e) {
            DB::rollBack();
            Log::error('TmsInbound@push failed', ['error' => $e->getMessage(), 'tms_load_id' => $request->input('tms_load_id')]);

            // If handoff was created before the error, mark it failed
            if (isset($handoff) && $handoff->exists) {
                $handoff->markFailed($e->getMessage());
                $handoff->recordEvent('push_failed', $request->input('pushed_by'), $request->input('source_module'), $e->getMessage());
            }

            return response()->json([
                'status'  => 'push_failed',
                'message' => 'Failed to publish load to board.',
            ], 500);
        }
    }

    /**
     * POST /api/stloads/queue
     *
     * Accept a load payload from TMS and store it for later processing.
     * Does NOT create a Load record yet.
     */
    public function queue(Request $request): JsonResponse
    {
        $validator = $this->validatePayload($request);

        if ($validator->fails()) {
            return response()->json([
                'status'  => 'validation_error',
                'errors'  => $validator->errors(),
            ], 422);
        }

        try {
            $handoff = $this->createHandoff($request, StloadsHandoff::STATUS_QUEUED);
            $handoff->recordEvent('queued', $request->input('pushed_by'), $request->input('source_module'));

            return response()->json([
                'status'     => 'queued',
                'handoff_id' => $handoff->id,
            ], 201);
        } catch (\Throwable $e) {
            Log::error('TmsInbound@queue failed', ['error' => $e->getMessage(), 'tms_load_id' => $request->input('tms_load_id')]);

            return response()->json([
                'status'  => 'queue_failed',
                'message' => 'Failed to queue handoff.',
            ], 500);
        }
    }

    /**
     * POST /api/stloads/requeue
     *
     * Retry a previously failed or requeue-required handoff.
     */
    public function requeue(Request $request): JsonResponse
    {
        $request->validate([
            'handoff_id' => ['required', 'integer', 'exists:stloads_handoffs,id'],
        ]);

        $handoff = StloadsHandoff::findOrFail($request->input('handoff_id'));

        if (!in_array($handoff->status, [StloadsHandoff::STATUS_PUSH_FAILED, StloadsHandoff::STATUS_REQUEUE_REQUIRED])) {
            return response()->json([
                'status'  => 'invalid_state',
                'message' => "Handoff #{$handoff->id} is in status '{$handoff->status}' and cannot be requeued.",
            ], 409);
        }

        try {
            DB::beginTransaction();

            $handoff->update(['status' => StloadsHandoff::STATUS_PUSH_IN_PROGRESS]);
            $handoff->recordEvent('requeue_started', $request->input('pushed_by'), $request->input('source_module'));

            // Re-hydrate a request-like object from the stored raw payload
            $payload = $handoff->raw_payload;
            $fakeRequest = new Request($payload);

            $load = $this->materializeLoad($fakeRequest, $handoff);

            $handoff->markPublished($load->id);
            $handoff->recordEvent('published', $request->input('pushed_by'), $request->input('source_module'), 'success');

            DB::commit();

            return response()->json([
                'status'      => 'published',
                'handoff_id'  => $handoff->id,
                'load_id'     => $load->id,
                'load_number' => $load->load_number,
            ]);
        } catch (\Throwable $e) {
            DB::rollBack();
            Log::error('TmsInbound@requeue failed', ['error' => $e->getMessage(), 'handoff_id' => $handoff->id]);

            $handoff->markFailed($e->getMessage());
            $handoff->recordEvent('requeue_failed', $request->input('pushed_by'), $request->input('source_module'), $e->getMessage());

            return response()->json([
                'status'  => 'push_failed',
                'message' => 'Requeue attempt failed.',
            ], 500);
        }
    }

    // ── Private helpers ───────────────────────────────────────

    private function validatePayload(Request $request): \Illuminate\Validation\Validator
    {
        return Validator::make($request->all(), [
            // Identity
            'tms_load_id'          => ['required', 'string', 'max:100'],
            'tenant_id'            => ['required', 'string', 'max:100'],
            'external_handoff_id'  => ['nullable', 'string', 'max:100'],

            // Classification
            'party_type'           => ['required', 'string', 'in:shipper,broker,freight_forwarder'],
            'freight_mode'         => ['required', 'string', 'max:50'],
            'equipment_type'       => ['required', 'string', 'max:100'],
            'commodity_description'=> ['nullable', 'string', 'max:500'],
            'weight'               => ['required', 'numeric', 'min:0'],
            'weight_unit'          => ['required', 'string', 'in:LBS,KG,MTON'],
            'piece_count'          => ['nullable', 'integer', 'min:0'],

            // Cargo details
            'is_hazardous'         => ['nullable', 'boolean'],
            'temperature_data'     => ['nullable', 'array'],
            'container_data'       => ['nullable', 'array'],
            'securement_data'      => ['nullable', 'array'],

            // Pickup
            'pickup_city'          => ['required', 'string', 'max:255'],
            'pickup_state'         => ['nullable', 'string', 'max:255'],
            'pickup_zip'           => ['nullable', 'string', 'max:20'],
            'pickup_country'       => ['required', 'string', 'max:255'],
            'pickup_address'       => ['required', 'string', 'max:500'],
            'pickup_window_start'  => ['required', 'date'],
            'pickup_window_end'    => ['nullable', 'date', 'after_or_equal:pickup_window_start'],
            'pickup_instructions'  => ['nullable', 'string', 'max:2000'],
            'pickup_appointment_ref' => ['nullable', 'string', 'max:100'],

            // Dropoff
            'dropoff_city'         => ['required', 'string', 'max:255'],
            'dropoff_state'        => ['nullable', 'string', 'max:255'],
            'dropoff_zip'          => ['nullable', 'string', 'max:20'],
            'dropoff_country'      => ['required', 'string', 'max:255'],
            'dropoff_address'      => ['required', 'string', 'max:500'],
            'dropoff_window_start' => ['required', 'date', 'after_or_equal:pickup_window_start'],
            'dropoff_window_end'   => ['nullable', 'date', 'after_or_equal:dropoff_window_start'],
            'dropoff_instructions' => ['nullable', 'string', 'max:2000'],
            'dropoff_appointment_ref' => ['nullable', 'string', 'max:100'],

            // Pricing
            'board_rate'           => ['required', 'numeric', 'min:0'],
            'rate_currency'        => ['nullable', 'string', 'max:10'],
            'bid_type'             => ['required', 'string', 'in:Fixed,Open'],

            // Compliance
            'compliance_passed'    => ['nullable', 'boolean'],
            'compliance_summary'   => ['nullable', 'array'],

            // Meta
            'pushed_by'            => ['nullable', 'string', 'max:255'],
            'push_reason'          => ['nullable', 'string', 'max:500'],
            'source_module'        => ['nullable', 'string', 'max:100'],
            'payload_version'      => ['nullable', 'string', 'max:20'],
        ]);
    }

    private function createHandoff(Request $request, string $status): StloadsHandoff
    {
        return StloadsHandoff::create([
            'tms_load_id'              => $request->input('tms_load_id'),
            'tenant_id'                => $request->input('tenant_id'),
            'external_handoff_id'      => $request->input('external_handoff_id'),
            'status'                   => $status,

            'party_type'               => $request->input('party_type'),
            'freight_mode'             => $request->input('freight_mode'),
            'equipment_type'           => $request->input('equipment_type'),
            'commodity_description'    => $request->input('commodity_description'),
            'weight'                   => $request->input('weight'),
            'weight_unit'              => $request->input('weight_unit'),
            'piece_count'              => $request->input('piece_count'),

            'is_hazardous'             => $request->boolean('is_hazardous'),
            'temperature_data'         => $request->input('temperature_data'),
            'container_data'           => $request->input('container_data'),
            'securement_data'          => $request->input('securement_data'),

            'pickup_city'              => $request->input('pickup_city'),
            'pickup_state'             => $request->input('pickup_state'),
            'pickup_zip'               => $request->input('pickup_zip'),
            'pickup_country'           => $request->input('pickup_country'),
            'pickup_address'           => $request->input('pickup_address'),
            'pickup_window_start'      => $request->input('pickup_window_start'),
            'pickup_window_end'        => $request->input('pickup_window_end'),
            'pickup_instructions'      => $request->input('pickup_instructions'),
            'pickup_appointment_ref'   => $request->input('pickup_appointment_ref'),

            'dropoff_city'             => $request->input('dropoff_city'),
            'dropoff_state'            => $request->input('dropoff_state'),
            'dropoff_zip'              => $request->input('dropoff_zip'),
            'dropoff_country'          => $request->input('dropoff_country'),
            'dropoff_address'          => $request->input('dropoff_address'),
            'dropoff_window_start'     => $request->input('dropoff_window_start'),
            'dropoff_window_end'       => $request->input('dropoff_window_end'),
            'dropoff_instructions'     => $request->input('dropoff_instructions'),
            'dropoff_appointment_ref'  => $request->input('dropoff_appointment_ref'),

            'board_rate'               => $request->input('board_rate'),
            'rate_currency'            => $request->input('rate_currency', 'USD'),
            'accessorial_flags'        => $request->input('accessorial_flags'),
            'bid_type'                 => $request->input('bid_type'),
            'quote_status'             => $request->input('quote_status'),
            'tender_posture'           => $request->input('tender_posture'),

            'compliance_passed'        => $request->boolean('compliance_passed'),
            'compliance_summary'       => $request->input('compliance_summary'),
            'required_documents_status'=> $request->input('required_documents_status'),
            'readiness'                => $request->input('readiness'),

            'pushed_by'                => $request->input('pushed_by'),
            'push_reason'              => $request->input('push_reason'),
            'source_module'            => $request->input('source_module'),
            'payload_version'          => $request->input('payload_version'),
            'raw_payload'              => $request->all(),

            'queued_at'                => $status === StloadsHandoff::STATUS_QUEUED ? now() : null,
        ]);
    }

    /**
     * Convert a handoff payload into a real Load + LoadLeg + Locations.
     */
    private function materializeLoad(Request $request, StloadsHandoff $handoff): Load
    {
        // Resolve equipment by name → id
        $equipment = Equipments::where('name', 'LIKE', '%' . $request->input('equipment_type') . '%')->first();

        // Resolve load type from freight_mode
        $loadType = LoadType::where('name', 'LIKE', '%' . $request->input('freight_mode') . '%')->first();

        // Map party_type to role id for load number generation
        $roleId = match ($request->input('party_type')) {
            'shipper'            => 2,
            'broker'             => 4,
            'freight_forwarder'  => 5,
            default              => 2,
        };

        $loadNumber = LoadNumbers::generateLoadNumber($roleId);

        // Create the Load record
        $load = Load::create([
            'load_number'              => $loadNumber,
            'title'                    => $this->buildLoadTitle($request),
            'load_type_id'             => $loadType?->id,
            'equipment_id'             => $equipment?->id,
            'commodity_type_id'        => null, // TMS sends description, not id
            'weight_unit'              => $request->input('weight_unit'),
            'weight'                   => $request->input('weight'),
            'special_instructions'     => $request->input('pickup_instructions'),
            'is_hazardous'             => $request->boolean('is_hazardous'),
            'is_temperature_controlled'=> !empty($request->input('temperature_data')),
            'user_id'                  => null, // TMS-originated, no web user
            'status'                   => 1,
            'leg_count'                => 1,
        ]);

        // Create pickup location
        $pickupCountry = Country::where('name', 'LIKE', '%' . $request->input('pickup_country') . '%')->first();
        $pickupCity = null;
        if ($pickupCountry) {
            $pickupCity = City::firstOrCreate(
                ['name' => $request->input('pickup_city'), 'country_id' => $pickupCountry->id],
            );
        }

        $pickupLocation = Locations::create([
            'name'       => $request->input('pickup_address'),
            'city_id'    => $pickupCity?->id,
            'country_id' => $pickupCountry?->id,
        ]);

        // Create dropoff location
        $dropoffCountry = Country::where('name', 'LIKE', '%' . $request->input('dropoff_country') . '%')->first();
        $dropoffCity = null;
        if ($dropoffCountry) {
            $dropoffCity = City::firstOrCreate(
                ['name' => $request->input('dropoff_city'), 'country_id' => $dropoffCountry->id],
            );
        }

        $dropoffLocation = Locations::create([
            'name'       => $request->input('dropoff_address'),
            'city_id'    => $dropoffCity?->id,
            'country_id' => $dropoffCountry?->id,
        ]);

        // Create a single leg (TMS pushes one origin→destination per handoff)
        LoadLeg::create([
            'load_id'              => $load->id,
            'leg_no'               => 1,
            'leg_code'             => LoadNumbers::legCode($loadNumber, 1),
            'pickup_location_id'   => $pickupLocation->id,
            'delivery_location_id' => $dropoffLocation->id,
            'pickup_date'          => $request->input('pickup_window_start'),
            'delivery_date'        => $request->input('dropoff_window_start'),
            'bid_status'           => $request->input('bid_type', 'Open'),
            'price'                => $request->input('board_rate'),
            'status_id'            => 1,
        ]);

        return $load;
    }

    private function buildLoadTitle(Request $request): string
    {
        $origin = $request->input('pickup_city', 'Origin');
        $dest   = $request->input('dropoff_city', 'Destination');
        $mode   = $request->input('freight_mode', 'Load');

        return "{$mode}: {$origin} → {$dest}";
    }
}
