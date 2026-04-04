<?php

namespace App\Services;

use App\Models\StloadsHandoff;
use App\Models\StloadsSyncError;
use Illuminate\Support\Facades\Log;

class StloadsSyncMonitor
{
    /**
     * Record a push failure as a sync error.
     */
    public static function pushFailed(StloadsHandoff $handoff, string $errorMessage, ?string $performedBy = null): StloadsSyncError
    {
        $error = StloadsSyncError::create([
            'handoff_id'    => $handoff->id,
            'error_class'   => StloadsSyncError::CLASS_PUSH_FAILED,
            'severity'      => StloadsSyncError::SEVERITY_ERROR,
            'title'         => "Push failed for TMS load {$handoff->tms_load_id}",
            'detail'        => $errorMessage,
            'source_module' => $handoff->source_module,
            'performed_by'  => $performedBy ?? $handoff->pushed_by,
        ]);

        Log::channel('stack')->warning('[STLOADS SYNC] Push failed', [
            'handoff_id'  => $handoff->id,
            'tms_load_id' => $handoff->tms_load_id,
            'error'       => $errorMessage,
        ]);

        return $error;
    }

    /**
     * Detect and record duplicate publish attempts.
     * Returns the existing handoff if a duplicate is found, null otherwise.
     */
    public static function checkDuplicatePublish(string $tmsLoadId, string $tenantId): ?StloadsHandoff
    {
        $existing = StloadsHandoff::where('tms_load_id', $tmsLoadId)
            ->where('tenant_id', $tenantId)
            ->where('status', StloadsHandoff::STATUS_PUBLISHED)
            ->first();

        if ($existing) {
            StloadsSyncError::create([
                'handoff_id'    => $existing->id,
                'error_class'   => StloadsSyncError::CLASS_DUPLICATE_PUBLISH,
                'severity'      => StloadsSyncError::SEVERITY_WARNING,
                'title'         => "Duplicate publish attempt for TMS load {$tmsLoadId}",
                'detail'        => "Handoff #{$existing->id} is already published for this TMS load. The new push was rejected.",
                'source_module' => $existing->source_module,
            ]);

            Log::channel('stack')->warning('[STLOADS SYNC] Duplicate publish detected', [
                'tms_load_id'        => $tmsLoadId,
                'existing_handoff'   => $existing->id,
            ]);
        }

        return $existing;
    }

    /**
     * Record a compliance-blocked suppression.
     */
    public static function complianceBlocked(array $payload, array $blockers): StloadsSyncError
    {
        $handoffId = null;

        // Try to find the handoff if it was already queued
        if (!empty($payload['tms_load_id']) && !empty($payload['tenant_id'])) {
            $handoff = StloadsHandoff::where('tms_load_id', $payload['tms_load_id'])
                ->where('tenant_id', $payload['tenant_id'])
                ->latest()
                ->first();
            $handoffId = $handoff?->id;
        }

        $error = StloadsSyncError::create([
            'handoff_id'    => $handoffId,
            'error_class'   => StloadsSyncError::CLASS_COMPLIANCE_BLOCKED,
            'severity'      => StloadsSyncError::SEVERITY_WARNING,
            'title'         => "Compliance blocked for TMS load " . ($payload['tms_load_id'] ?? 'unknown'),
            'detail'        => "Release gate blockers: " . implode('; ', $blockers),
            'source_module' => $payload['source_module'] ?? null,
            'performed_by'  => $payload['pushed_by'] ?? null,
        ]);

        Log::channel('stack')->warning('[STLOADS SYNC] Compliance blocked', [
            'tms_load_id' => $payload['tms_load_id'] ?? 'unknown',
            'blockers'    => $blockers,
        ]);

        return $error;
    }

    /**
     * Record a release gate failure (non-compliance structural issues).
     */
    public static function releaseGateFailed(array $payload, array $blockers): StloadsSyncError
    {
        $error = StloadsSyncError::create([
            'handoff_id'    => null,
            'error_class'   => StloadsSyncError::CLASS_RELEASE_GATE_FAILED,
            'severity'      => StloadsSyncError::SEVERITY_WARNING,
            'title'         => "Release gate failed for TMS load " . ($payload['tms_load_id'] ?? 'unknown'),
            'detail'        => "Blockers: " . implode('; ', $blockers),
            'source_module' => $payload['source_module'] ?? null,
            'performed_by'  => $payload['pushed_by'] ?? null,
        ]);

        Log::channel('stack')->info('[STLOADS SYNC] Release gate failed', [
            'tms_load_id' => $payload['tms_load_id'] ?? 'unknown',
            'blockers'    => $blockers,
        ]);

        return $error;
    }

    /**
     * Record a payload validation failure.
     */
    public static function payloadValidationFailed(array $payload, array $errors): StloadsSyncError
    {
        return StloadsSyncError::create([
            'handoff_id'    => null,
            'error_class'   => StloadsSyncError::CLASS_PAYLOAD_VALIDATION,
            'severity'      => StloadsSyncError::SEVERITY_WARNING,
            'title'         => "Payload validation failed for TMS load " . ($payload['tms_load_id'] ?? 'unknown'),
            'detail'        => json_encode($errors),
            'source_module' => $payload['source_module'] ?? null,
            'performed_by'  => $payload['pushed_by'] ?? null,
        ]);
    }

    /**
     * Scan for published handoffs whose linked load has been delivered/completed
     * internally but the handoff is still open (not closed/withdrawn).
     *
     * Returns the count of newly created warnings.
     */
    public static function detectDeliveredStillOpen(): int
    {
        // Status 11 = delivered/completed in LoadLeg status_master
        $openHandoffs = StloadsHandoff::where('status', StloadsHandoff::STATUS_PUBLISHED)
            ->whereNotNull('load_id')
            ->get();

        $count = 0;

        foreach ($openHandoffs as $handoff) {
            $allLegsCompleted = $handoff->load
                && $handoff->load->legs()->count() > 0
                && $handoff->load->legs()->where('status_id', '<', 10)->doesntExist();

            if ($allLegsCompleted) {
                // Don't duplicate — check if we already flagged this
                $alreadyFlagged = StloadsSyncError::where('handoff_id', $handoff->id)
                    ->where('error_class', StloadsSyncError::CLASS_DELIVERED_STILL_OPEN)
                    ->unresolved()
                    ->exists();

                if (!$alreadyFlagged) {
                    StloadsSyncError::create([
                        'handoff_id'  => $handoff->id,
                        'error_class' => StloadsSyncError::CLASS_DELIVERED_STILL_OPEN,
                        'severity'    => StloadsSyncError::SEVERITY_WARNING,
                        'title'       => "Load delivered but STLOADS handoff still open for TMS load {$handoff->tms_load_id}",
                        'detail'      => "All legs for load #{$handoff->load_id} are completed (status >= 10) but handoff #{$handoff->id} is still published. Consider closing or withdrawing.",
                    ]);
                    $count++;
                }
            }
        }

        if ($count > 0) {
            Log::channel('stack')->warning("[STLOADS SYNC] Found {$count} delivered-but-still-open handoffs");
        }

        return $count;
    }

    /**
     * Record a withdraw mismatch (e.g. internally withdrawn but externally still live).
     */
    public static function withdrawMismatch(StloadsHandoff $handoff, string $detail): StloadsSyncError
    {
        return StloadsSyncError::create([
            'handoff_id'  => $handoff->id,
            'error_class' => StloadsSyncError::CLASS_WITHDRAW_MISMATCH,
            'severity'    => StloadsSyncError::SEVERITY_ERROR,
            'title'       => "Withdraw mismatch for TMS load {$handoff->tms_load_id}",
            'detail'      => $detail,
        ]);
    }

    /**
     * Store external references from TMS for a handoff.
     */
    public static function storeExternalRefs(StloadsHandoff $handoff, array $refs): void
    {
        foreach ($refs as $ref) {
            if (!empty($ref['type']) && !empty($ref['value'])) {
                $handoff->externalRefs()->create([
                    'ref_type'   => $ref['type'],
                    'ref_value'  => $ref['value'],
                    'ref_source' => $ref['source'] ?? null,
                ]);
            }
        }
    }
}
