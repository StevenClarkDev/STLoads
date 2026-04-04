<?php

namespace App\Http\Controllers\Api;

use App\Http\Controllers\Controller;
use App\Models\StloadsHandoff;
use App\Services\StloadsReconciler;
use App\Services\StloadsSyncMonitor;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\Validator;

class TmsWebhookController extends Controller
{
    /**
     * POST /api/stloads/webhook/status
     *
     * Receive status-update webhooks from TMS dispatch system.
     * This is the primary mechanism for keeping STLOADS in sync with TMS.
     */
    public function status(Request $request): JsonResponse
    {
        $validator = Validator::make($request->all(), [
            'tms_load_id'    => ['required', 'string', 'max:100'],
            'tenant_id'      => ['required', 'string', 'max:100'],
            'tms_status'     => ['required', 'string', 'in:dispatched,in_transit,at_pickup,at_delivery,delivered,cancelled,invoiced,settled'],
            'status_at'      => ['nullable', 'date'],
            'source_module'  => ['nullable', 'string', 'max:100'],
            'pushed_by'      => ['nullable', 'string', 'max:255'],
            'detail'         => ['nullable', 'string', 'max:2000'],
            'rate_update'    => ['nullable', 'numeric', 'min:0'],
        ]);

        if ($validator->fails()) {
            return response()->json([
                'status' => 'validation_error',
                'errors' => $validator->errors(),
            ], 422);
        }

        // Find the handoff
        $handoff = StloadsHandoff::where('tms_load_id', $request->input('tms_load_id'))
            ->where('tenant_id', $request->input('tenant_id'))
            ->whereNotIn('status', [StloadsHandoff::STATUS_CLOSED])
            ->latest()
            ->first();

        if (!$handoff) {
            Log::info('[STLOADS WEBHOOK] No active handoff for status update', [
                'tms_load_id' => $request->input('tms_load_id'),
                'tenant_id'   => $request->input('tenant_id'),
            ]);

            return response()->json([
                'status'  => 'no_handoff',
                'message' => 'No active handoff found for this TMS load.',
            ], 404);
        }

        // Process the status update through the reconciler
        $log = StloadsReconciler::processStatusUpdate(
            $handoff,
            $request->input('tms_status'),
            $request->all(),
            $request->input('pushed_by', 'webhook')
        );

        // Handle rate update if included
        if ($request->has('rate_update') && $request->input('rate_update') > 0) {
            $this->processRateUpdate($handoff, $request->input('rate_update'), $request->all());
        }

        return response()->json([
            'status'           => 'accepted',
            'handoff_id'       => $handoff->id,
            'tms_status'       => $handoff->fresh()->tms_status,
            'stloads_status'   => $handoff->fresh()->status,
            'reconciliation_id'=> $log->id,
        ]);
    }

    /**
     * POST /api/stloads/webhook/bulk-status
     *
     * Accept multiple status updates in a single call (batch reconciliation).
     */
    public function bulkStatus(Request $request): JsonResponse
    {
        $request->validate([
            'updates'              => ['required', 'array', 'min:1', 'max:100'],
            'updates.*.tms_load_id'=> ['required', 'string', 'max:100'],
            'updates.*.tenant_id'  => ['required', 'string', 'max:100'],
            'updates.*.tms_status' => ['required', 'string', 'in:dispatched,in_transit,at_pickup,at_delivery,delivered,cancelled,invoiced,settled'],
            'updates.*.pushed_by'  => ['nullable', 'string', 'max:255'],
        ]);

        $results = [
            'processed' => 0,
            'skipped'   => 0,
            'errors'    => 0,
            'details'   => [],
        ];

        foreach ($request->input('updates') as $update) {
            $handoff = StloadsHandoff::where('tms_load_id', $update['tms_load_id'])
                ->where('tenant_id', $update['tenant_id'])
                ->whereNotIn('status', [StloadsHandoff::STATUS_CLOSED])
                ->latest()
                ->first();

            if (!$handoff) {
                $results['skipped']++;
                $results['details'][] = [
                    'tms_load_id' => $update['tms_load_id'],
                    'result'      => 'no_handoff',
                ];
                continue;
            }

            try {
                StloadsReconciler::processStatusUpdate(
                    $handoff,
                    $update['tms_status'],
                    $update,
                    $update['pushed_by'] ?? 'webhook_bulk'
                );
                $results['processed']++;
                $results['details'][] = [
                    'tms_load_id' => $update['tms_load_id'],
                    'handoff_id'  => $handoff->id,
                    'result'      => 'processed',
                ];
            } catch (\Throwable $e) {
                $results['errors']++;
                $results['details'][] = [
                    'tms_load_id' => $update['tms_load_id'],
                    'result'      => 'error',
                    'message'     => $e->getMessage(),
                ];
                Log::error('[STLOADS WEBHOOK] Bulk status error', [
                    'tms_load_id' => $update['tms_load_id'],
                    'error'       => $e->getMessage(),
                ]);
            }
        }

        return response()->json(array_merge(['status' => 'completed'], $results));
    }

    /**
     * POST /api/stloads/webhook/cancel
     *
     * Explicit cancellation webhook — TMS says "this load is cancelled, pull it down."
     */
    public function cancel(Request $request): JsonResponse
    {
        $request->validate([
            'tms_load_id' => ['required', 'string', 'max:100'],
            'tenant_id'   => ['required', 'string', 'max:100'],
            'reason'      => ['nullable', 'string', 'max:500'],
            'pushed_by'   => ['nullable', 'string', 'max:255'],
        ]);

        $handoff = StloadsHandoff::where('tms_load_id', $request->input('tms_load_id'))
            ->where('tenant_id', $request->input('tenant_id'))
            ->whereNotIn('status', [StloadsHandoff::STATUS_CLOSED, StloadsHandoff::STATUS_WITHDRAWN])
            ->latest()
            ->first();

        if (!$handoff) {
            return response()->json([
                'status'  => 'no_handoff',
                'message' => 'No active handoff found to cancel.',
            ], 404);
        }

        // Update TMS status tracking
        $handoff->update([
            'tms_status'      => StloadsHandoff::TMS_CANCELLED,
            'tms_status_at'   => now(),
            'last_webhook_at' => now(),
        ]);

        // Auto-withdraw from the board
        $result = StloadsReconciler::autoWithdraw(
            $handoff,
            $request->input('reason', 'TMS cancellation webhook'),
            $request->input('pushed_by', 'webhook')
        );

        return response()->json([
            'status'         => $result ? 'withdrawn' : 'already_inactive',
            'handoff_id'     => $handoff->id,
            'stloads_status' => $handoff->fresh()->status,
        ]);
    }

    /**
     * POST /api/stloads/webhook/close
     *
     * Archive webhook — TMS says "this dispatch is complete, close everything."
     */
    public function archiveClose(Request $request): JsonResponse
    {
        $request->validate([
            'tms_load_id' => ['required', 'string', 'max:100'],
            'tenant_id'   => ['required', 'string', 'max:100'],
            'reason'      => ['nullable', 'string', 'max:500'],
            'pushed_by'   => ['nullable', 'string', 'max:255'],
        ]);

        $handoff = StloadsHandoff::where('tms_load_id', $request->input('tms_load_id'))
            ->where('tenant_id', $request->input('tenant_id'))
            ->where('status', '!=', StloadsHandoff::STATUS_CLOSED)
            ->latest()
            ->first();

        if (!$handoff) {
            return response()->json([
                'status'  => 'no_handoff',
                'message' => 'No open handoff found to archive.',
            ], 404);
        }

        $result = StloadsReconciler::autoArchive(
            $handoff,
            $request->input('reason', 'TMS archive/close webhook'),
            $request->input('pushed_by', 'webhook')
        );

        return response()->json([
            'status'         => $result ? 'closed' : 'already_closed',
            'handoff_id'     => $handoff->id,
            'stloads_status' => $handoff->fresh()->status,
        ]);
    }

    // ── Private helpers ───────────────────────────────────────

    private function processRateUpdate(StloadsHandoff $handoff, float $newRate, array $webhookPayload): void
    {
        $oldRate = $handoff->board_rate;

        if (abs($oldRate - $newRate) < 0.01) {
            return; // No meaningful change
        }

        $handoff->update(['board_rate' => $newRate]);

        // If there's a linked load, update the leg price too
        if ($handoff->load_id) {
            $load = $handoff->load;
            if ($load) {
                $load->legs()->update(['price' => $newRate]);
            }
        }

        $handoff->recordEvent('rate_updated', $webhookPayload['pushed_by'] ?? 'webhook', null, "\${$oldRate} → \${$newRate}");

        \App\Models\StloadsReconciliationLog::create([
            'handoff_id'      => $handoff->id,
            'action'          => \App\Models\StloadsReconciliationLog::ACTION_RATE_UPDATE,
            'detail'          => "Rate updated from \${$oldRate} to \${$newRate}",
            'triggered_by'    => $webhookPayload['pushed_by'] ?? 'webhook',
            'webhook_payload' => $webhookPayload,
        ]);
    }
}
