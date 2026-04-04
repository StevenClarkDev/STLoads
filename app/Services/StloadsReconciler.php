<?php

namespace App\Services;

use App\Models\Load;
use App\Models\StloadsHandoff;
use App\Models\StloadsReconciliationLog;
use App\Models\StloadsSyncError;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;

class StloadsReconciler
{
    /**
     * Process an inbound status webhook from TMS.
     *
     * @param  StloadsHandoff  $handoff    The matched handoff record
     * @param  string          $newTmsStatus  The new TMS-side lifecycle status
     * @param  array           $webhookPayload  Raw webhook body for audit
     * @param  string|null     $triggeredBy  'webhook', 'cron', or user email
     * @return StloadsReconciliationLog
     */
    public static function processStatusUpdate(
        StloadsHandoff $handoff,
        string $newTmsStatus,
        array  $webhookPayload = [],
        ?string $triggeredBy = 'webhook'
    ): StloadsReconciliationLog {
        $oldTmsStatus    = $handoff->tms_status;
        $oldStloadsStatus = $handoff->status;

        // Update the TMS status tracking
        $handoff->update([
            'tms_status'      => $newTmsStatus,
            'tms_status_at'   => now(),
            'last_webhook_at' => now(),
        ]);

        // Record event
        $handoff->recordEvent(
            'tms_status_update',
            $triggeredBy,
            $webhookPayload['source_module'] ?? null,
            "{$oldTmsStatus} → {$newTmsStatus}"
        );

        // Determine if STLOADS-side action is needed
        $stloadsStatusTo = null;

        switch ($newTmsStatus) {
            case StloadsHandoff::TMS_CANCELLED:
                // TMS cancelled → auto-withdraw from STLOADS board
                $stloadsStatusTo = self::autoWithdraw($handoff, "TMS dispatch cancelled ({$triggeredBy})", $triggeredBy);
                break;

            case StloadsHandoff::TMS_DELIVERED:
                // Delivered → flag if handoff still published (soft warning, don't auto-close yet)
                if ($handoff->status === StloadsHandoff::STATUS_PUBLISHED) {
                    StloadsSyncMonitor::detectDeliveredStillOpen();
                }
                break;

            case StloadsHandoff::TMS_INVOICED:
            case StloadsHandoff::TMS_SETTLED:
                // Financial milestone → auto-close + archive if still published/withdrawn
                $stloadsStatusTo = self::autoArchive($handoff, "TMS reached {$newTmsStatus} ({$triggeredBy})", $triggeredBy);
                break;
        }

        // Log the reconciliation action
        return StloadsReconciliationLog::create([
            'handoff_id'         => $handoff->id,
            'action'             => StloadsReconciliationLog::ACTION_STATUS_UPDATE,
            'tms_status_from'    => $oldTmsStatus,
            'tms_status_to'      => $newTmsStatus,
            'stloads_status_from'=> $oldStloadsStatus,
            'stloads_status_to'  => $stloadsStatusTo ?? $handoff->status,
            'detail'             => "TMS status updated: {$oldTmsStatus} → {$newTmsStatus}",
            'triggered_by'       => $triggeredBy,
            'webhook_payload'    => $webhookPayload,
        ]);
    }

    /**
     * Auto-withdraw a handoff: soft-delete its Load and mark withdrawn.
     * Only applies when published.
     */
    public static function autoWithdraw(StloadsHandoff $handoff, string $reason, ?string $triggeredBy = 'system'): ?string
    {
        if ($handoff->status !== StloadsHandoff::STATUS_PUBLISHED) {
            // Already withdrawn or closed; skip
            return null;
        }

        try {
            DB::beginTransaction();

            if ($handoff->load_id) {
                Load::find($handoff->load_id)?->delete();
            }

            $handoff->update([
                'status'       => StloadsHandoff::STATUS_WITHDRAWN,
                'withdrawn_at' => now(),
            ]);

            $handoff->recordEvent('auto_withdrawn', $triggeredBy, null, $reason);

            StloadsReconciliationLog::create([
                'handoff_id'          => $handoff->id,
                'action'              => StloadsReconciliationLog::ACTION_AUTO_WITHDRAW,
                'stloads_status_from' => StloadsHandoff::STATUS_PUBLISHED,
                'stloads_status_to'   => StloadsHandoff::STATUS_WITHDRAWN,
                'detail'              => $reason,
                'triggered_by'        => $triggeredBy,
            ]);

            DB::commit();

            Log::channel('stack')->info('[STLOADS RECONCILER] Auto-withdrawn', [
                'handoff_id'  => $handoff->id,
                'tms_load_id' => $handoff->tms_load_id,
                'reason'      => $reason,
            ]);

            return StloadsHandoff::STATUS_WITHDRAWN;
        } catch (\Throwable $e) {
            DB::rollBack();
            Log::error('[STLOADS RECONCILER] Auto-withdraw failed', [
                'handoff_id' => $handoff->id,
                'error'      => $e->getMessage(),
            ]);
            StloadsSyncMonitor::withdrawMismatch($handoff, "Auto-withdraw failed: {$e->getMessage()}");
            return null;
        }
    }

    /**
     * Auto-archive (close) a handoff when TMS reaches a terminal financial status.
     * Works on published or withdrawn handoffs.
     */
    public static function autoArchive(StloadsHandoff $handoff, string $reason, ?string $triggeredBy = 'system'): ?string
    {
        $archiveable = [
            StloadsHandoff::STATUS_PUBLISHED,
            StloadsHandoff::STATUS_WITHDRAWN,
        ];

        if (!in_array($handoff->status, $archiveable)) {
            return null;
        }

        $oldStatus = $handoff->status;

        try {
            DB::beginTransaction();

            // If still published, soft-delete the load first
            if ($handoff->status === StloadsHandoff::STATUS_PUBLISHED && $handoff->load_id) {
                Load::find($handoff->load_id)?->delete();
            }

            $handoff->update([
                'status'    => StloadsHandoff::STATUS_CLOSED,
                'closed_at' => now(),
            ]);

            $handoff->recordEvent('auto_archived', $triggeredBy, null, $reason);

            StloadsReconciliationLog::create([
                'handoff_id'          => $handoff->id,
                'action'              => StloadsReconciliationLog::ACTION_AUTO_ARCHIVE,
                'stloads_status_from' => $oldStatus,
                'stloads_status_to'   => StloadsHandoff::STATUS_CLOSED,
                'detail'              => $reason,
                'triggered_by'        => $triggeredBy,
            ]);

            DB::commit();

            Log::channel('stack')->info('[STLOADS RECONCILER] Auto-archived', [
                'handoff_id'  => $handoff->id,
                'tms_load_id' => $handoff->tms_load_id,
            ]);

            return StloadsHandoff::STATUS_CLOSED;
        } catch (\Throwable $e) {
            DB::rollBack();
            Log::error('[STLOADS RECONCILER] Auto-archive failed', [
                'handoff_id' => $handoff->id,
                'error'      => $e->getMessage(),
            ]);
            return null;
        }
    }

    /**
     * Bulk reconciliation scan — detect mismatches between TMS and STLOADS states.
     * Designed to run as a scheduled cron job.
     *
     * Returns summary counts.
     */
    public static function runReconciliationScan(): array
    {
        $results = [
            'delivered_still_open'  => 0,
            'cancelled_still_live'  => 0,
            'stale_handoffs'        => 0,
            'auto_archived'         => 0,
        ];

        // 1. Delivered/invoiced/settled on TMS but still published on STLOADS
        $terminalTmsStatuses = [
            StloadsHandoff::TMS_INVOICED,
            StloadsHandoff::TMS_SETTLED,
        ];

        $stalePublished = StloadsHandoff::where('status', StloadsHandoff::STATUS_PUBLISHED)
            ->whereIn('tms_status', $terminalTmsStatuses)
            ->get();

        foreach ($stalePublished as $handoff) {
            self::autoArchive($handoff, "Reconciliation scan: TMS is {$handoff->tms_status} but STLOADS still published", 'cron');
            $results['auto_archived']++;
        }

        // 2. TMS cancelled but STLOADS still published
        $cancelledStillLive = StloadsHandoff::where('status', StloadsHandoff::STATUS_PUBLISHED)
            ->where('tms_status', StloadsHandoff::TMS_CANCELLED)
            ->get();

        foreach ($cancelledStillLive as $handoff) {
            self::autoWithdraw($handoff, 'Reconciliation scan: TMS cancelled but STLOADS still published', 'cron');
            $results['cancelled_still_live']++;
        }

        // 3. Delivered-still-open detection (uses existing sync monitor)
        $results['delivered_still_open'] = StloadsSyncMonitor::detectDeliveredStillOpen();

        // 4. Stale handoffs — published for >30 days with no webhook activity
        $staleThreshold = now()->subDays(30);
        $staleHandoffs = StloadsHandoff::where('status', StloadsHandoff::STATUS_PUBLISHED)
            ->where(function ($q) use ($staleThreshold) {
                $q->where('last_webhook_at', '<', $staleThreshold)
                  ->orWhereNull('last_webhook_at');
            })
            ->where('published_at', '<', $staleThreshold)
            ->get();

        foreach ($staleHandoffs as $handoff) {
            $alreadyFlagged = StloadsSyncError::where('handoff_id', $handoff->id)
                ->where('error_class', 'stale_handoff')
                ->unresolved()
                ->exists();

            if (!$alreadyFlagged) {
                StloadsSyncError::create([
                    'handoff_id'  => $handoff->id,
                    'error_class' => 'stale_handoff',
                    'severity'    => StloadsSyncError::SEVERITY_WARNING,
                    'title'       => "Stale handoff: no TMS activity for 30+ days (TMS load {$handoff->tms_load_id})",
                    'detail'      => "Published at {$handoff->published_at}, last webhook at " . ($handoff->last_webhook_at ?? 'never') . ". Consider closing or investigating.",
                ]);
                $results['stale_handoffs']++;
            }
        }

        if (array_sum($results) > 0) {
            Log::channel('stack')->info('[STLOADS RECONCILER] Scan complete', $results);
        }

        return $results;
    }

    /**
     * Force-sync a handoff — operator manually triggers state alignment.
     */
    public static function forceSync(StloadsHandoff $handoff, string $targetStatus, string $reason, string $performedBy): StloadsReconciliationLog
    {
        $oldStatus = $handoff->status;

        // Perform the status transition
        $updateData = ['status' => $targetStatus];

        if ($targetStatus === StloadsHandoff::STATUS_WITHDRAWN) {
            $updateData['withdrawn_at'] = now();
            if ($handoff->load_id) {
                Load::find($handoff->load_id)?->delete();
            }
        } elseif ($targetStatus === StloadsHandoff::STATUS_CLOSED) {
            $updateData['closed_at'] = now();
            if ($handoff->status === StloadsHandoff::STATUS_PUBLISHED && $handoff->load_id) {
                Load::find($handoff->load_id)?->delete();
            }
        }

        $handoff->update($updateData);
        $handoff->recordEvent('force_synced', $performedBy, null, "{$oldStatus} → {$targetStatus}: {$reason}");

        return StloadsReconciliationLog::create([
            'handoff_id'          => $handoff->id,
            'action'              => StloadsReconciliationLog::ACTION_FORCE_SYNC,
            'stloads_status_from' => $oldStatus,
            'stloads_status_to'   => $targetStatus,
            'detail'              => $reason,
            'triggered_by'        => $performedBy,
        ]);
    }
}
