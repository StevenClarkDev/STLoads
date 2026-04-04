<?php

namespace App\Http\Controllers;

use App\Models\StloadsHandoff;
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

        return view('stloads.operations', compact('handoffs', 'counts', 'statusFilter'));
    }

    public function show(StloadsHandoff $handoff)
    {
        $handoff->load(['load', 'events' => function ($q) {
            $q->latest();
        }]);

        return view('stloads.handoff_detail', compact('handoff'));
    }
}
