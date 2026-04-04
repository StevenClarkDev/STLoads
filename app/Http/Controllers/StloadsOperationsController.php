<?php

namespace App\Http\Controllers;

use App\Models\StloadsHandoff;
use App\Models\StloadsReconciliationLog;
use App\Models\StloadsSyncError;
use App\Services\StloadsReconciler;
use Illuminate\Http\Request;

class StloadsOperationsController extends Controller
{
    public function index(Request $request)
    {
        $statusFilter = $request->input('status');

        $query = StloadsHandoff::with('load')->latest();

        if ($statusFilter) {
            $query->where('status', $statusFilter);
        }

        $handoffs = $query->paginate(25);

        // Summary counts
        $counts = [
            'queued'           => StloadsHandoff::where('status', StloadsHandoff::STATUS_QUEUED)->count(),
            'push_in_progress' => StloadsHandoff::where('status', StloadsHandoff::STATUS_PUSH_IN_PROGRESS)->count(),
            'published'        => StloadsHandoff::where('status', StloadsHandoff::STATUS_PUBLISHED)->count(),
            'push_failed'      => StloadsHandoff::where('status', StloadsHandoff::STATUS_PUSH_FAILED)->count(),
            'requeue_required' => StloadsHandoff::where('status', StloadsHandoff::STATUS_REQUEUE_REQUIRED)->count(),
            'withdrawn'        => StloadsHandoff::where('status', StloadsHandoff::STATUS_WITHDRAWN)->count(),
            'closed'           => StloadsHandoff::where('status', StloadsHandoff::STATUS_CLOSED)->count(),
        ];

        // Unresolved sync errors for alert banner
        $syncErrors = StloadsSyncError::unresolved()
            ->latest()
            ->limit(10)
            ->get();

        $syncErrorCounts = [
            'total'    => StloadsSyncError::unresolved()->count(),
            'critical' => StloadsSyncError::unresolved()->bySeverity('critical')->count(),
            'error'    => StloadsSyncError::unresolved()->bySeverity('error')->count(),
            'warning'  => StloadsSyncError::unresolved()->bySeverity('warning')->count(),
        ];

        return view('stloads.operations', compact('handoffs', 'counts', 'statusFilter', 'syncErrors', 'syncErrorCounts'));
    }

    public function show(StloadsHandoff $handoff)
    {
        $handoff->load(['load', 'events' => function ($q) {
            $q->latest();
        }, 'externalRefs', 'syncErrors' => function ($q) {
            $q->latest();
        }, 'reconciliationLogs' => function ($q) {
            $q->latest();
        }]);

        return view('stloads.handoff_detail', compact('handoff'));
    }

    /**
     * POST /stloads/sync-error/{error}/resolve
     */
    public function resolveError(Request $request, StloadsSyncError $error)
    {
        $request->validate([
            'resolution_note' => ['nullable', 'string', 'max:500'],
        ]);

        $error->resolve(
            auth()->user()->name ?? auth()->user()->email,
            $request->input('resolution_note')
        );

        return back()->with('success', 'Sync error resolved.');
    }

    /**
     * GET /stloads/sync-errors
     */
    public function syncErrors(Request $request)
    {
        $query = StloadsSyncError::with('handoff')->latest();

        if ($request->input('resolved') === '0') {
            $query->unresolved();
        } elseif ($request->input('resolved') === '1') {
            $query->where('resolved', true);
        }

        if ($request->has('severity')) {
            $query->bySeverity($request->input('severity'));
        }

        $errors = $query->paginate(30);

        return view('stloads.sync_errors', compact('errors'));
    }

    /**
     * GET /stloads/reconciliation
     *
     * Exception dashboard — operational visibility for sync mismatches.
     */
    public function reconciliation(Request $request)
    {
        // Mismatch overview
        $published = StloadsHandoff::where('status', StloadsHandoff::STATUS_PUBLISHED);
        $mismatchCounts = [
            'total_published'       => (clone $published)->count(),
            'tms_cancelled'         => (clone $published)->where('tms_status', StloadsHandoff::TMS_CANCELLED)->count(),
            'tms_delivered'         => (clone $published)->where('tms_status', StloadsHandoff::TMS_DELIVERED)->count(),
            'tms_invoiced'          => (clone $published)->whereIn('tms_status', [StloadsHandoff::TMS_INVOICED, StloadsHandoff::TMS_SETTLED])->count(),
            'no_tms_status'         => (clone $published)->whereNull('tms_status')->count(),
            'stale_30d'             => (clone $published)->where(function ($q) {
                $q->where('last_webhook_at', '<', now()->subDays(30))
                  ->orWhereNull('last_webhook_at');
            })->where('published_at', '<', now()->subDays(30))->count(),
        ];

        // Sync error breakdown
        $errorBreakdown = StloadsSyncError::unresolved()
            ->selectRaw('error_class, severity, COUNT(*) as cnt')
            ->groupBy('error_class', 'severity')
            ->get()
            ->groupBy('error_class');

        // Recent reconciliation logs
        $recentLogs = StloadsReconciliationLog::with('handoff')
            ->latest()
            ->limit(25)
            ->get();

        // Filter for specific action type
        $actionFilter = $request->input('action');
        $logsQuery = StloadsReconciliationLog::with('handoff')->latest();
        if ($actionFilter) {
            $logsQuery->where('action', $actionFilter);
        }
        $logs = $logsQuery->paginate(30);

        return view('stloads.reconciliation', compact('mismatchCounts', 'errorBreakdown', 'recentLogs', 'logs', 'actionFilter'));
    }

    /**
     * POST /stloads/reconciliation/scan
     *
     * Triggerable reconciliation scan from UI.
     */
    public function runScan()
    {
        $results = StloadsReconciler::runReconciliationScan();

        $total = array_sum($results);
        $message = $total > 0
            ? "Reconciliation scan found {$total} items: " . implode(', ', array_map(fn($k, $v) => "{$k}={$v}", array_keys($results), $results))
            : 'Reconciliation scan complete — no issues found.';

        return back()->with('success', $message);
    }

    /**
     * POST /stloads/handoff/{handoff}/force-sync
     *
     * Operator-initiated force state alignment.
     */
    public function forceSync(Request $request, StloadsHandoff $handoff)
    {
        $request->validate([
            'target_status' => ['required', 'string', 'in:withdrawn,closed'],
            'reason'        => ['required', 'string', 'max:500'],
        ]);

        StloadsReconciler::forceSync(
            $handoff,
            $request->input('target_status'),
            $request->input('reason'),
            auth()->user()->name ?? auth()->user()->email
        );

        return back()->with('success', "Handoff #{$handoff->id} force-synced to {$request->input('target_status')}.");
    }
}
