<?php

namespace App\Http\Controllers;

use App\Models\StloadsHandoff;
use App\Models\StloadsSyncError;
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
}
