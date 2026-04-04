<?php

namespace App\Services;

use App\Models\StloadsHandoff;

class StloadsReleaseGate
{
    /**
     * Evaluate whether a handoff payload passes the minimum release gate.
     *
     * Returns ['pass' => true] or ['pass' => false, 'blockers' => [...]]
     */
    public static function evaluate(array $payload): array
    {
        $blockers = [];

        // 1. Valid tenant / account attached
        if (empty($payload['tenant_id'])) {
            $blockers[] = 'Missing tenant/account identifier (tenant_id).';
        }

        // 2. Valid party type
        $validParties = ['shipper', 'broker', 'freight_forwarder'];
        if (empty($payload['party_type']) || !in_array($payload['party_type'], $validParties)) {
            $blockers[] = 'Invalid or missing party type. Must be one of: ' . implode(', ', $validParties) . '.';
        }

        // 3. Valid freight mode
        if (empty($payload['freight_mode'])) {
            $blockers[] = 'Missing freight mode.';
        }

        // 4. Complete origin data
        if (empty($payload['pickup_city'])) {
            $blockers[] = 'Missing pickup city.';
        }
        if (empty($payload['pickup_country'])) {
            $blockers[] = 'Missing pickup country.';
        }
        if (empty($payload['pickup_address'])) {
            $blockers[] = 'Missing pickup address.';
        }

        // 5. Complete destination data
        if (empty($payload['dropoff_city'])) {
            $blockers[] = 'Missing dropoff city.';
        }
        if (empty($payload['dropoff_country'])) {
            $blockers[] = 'Missing dropoff country.';
        }
        if (empty($payload['dropoff_address'])) {
            $blockers[] = 'Missing dropoff address.';
        }

        // 6. Scheduled pickup present
        if (empty($payload['pickup_window_start'])) {
            $blockers[] = 'Missing scheduled pickup window start.';
        }

        // 7. Positive shipment weight
        if (!isset($payload['weight']) || (float) $payload['weight'] <= 0) {
            $blockers[] = 'Weight must be a positive number.';
        }

        // 8. Valid weight unit
        $validUnits = ['LBS', 'KG', 'MTON'];
        if (empty($payload['weight_unit']) || !in_array($payload['weight_unit'], $validUnits)) {
            $blockers[] = 'Invalid or missing weight unit. Must be one of: ' . implode(', ', $validUnits) . '.';
        }

        // 9. Equipment type present
        if (empty($payload['equipment_type'])) {
            $blockers[] = 'Missing equipment type.';
        }

        // 10. Board rate present and positive
        if (!isset($payload['board_rate']) || (float) $payload['board_rate'] <= 0) {
            $blockers[] = 'Board rate must be a positive number.';
        }

        // 11. Bid type present
        $validBidTypes = ['Fixed', 'Open'];
        if (empty($payload['bid_type']) || !in_array($payload['bid_type'], $validBidTypes)) {
            $blockers[] = 'Invalid or missing bid type. Must be Fixed or Open.';
        }

        // 12. No unresolved compliance blocker
        if (isset($payload['compliance_passed']) && $payload['compliance_passed'] === false) {
            $blockers[] = 'Compliance check has not passed. Load cannot be exposed to market.';
        }

        // 13. TMS load identity
        if (empty($payload['tms_load_id'])) {
            $blockers[] = 'Missing TMS load identifier (tms_load_id).';
        }

        return [
            'pass'     => empty($blockers),
            'blockers' => $blockers,
        ];
    }
}
