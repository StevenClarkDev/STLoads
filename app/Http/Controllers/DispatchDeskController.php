<?php

namespace App\Http\Controllers;

use App\Models\Load;
use App\Models\LoadLeg;
use App\Models\StloadsHandoff;
use App\Models\StloadsSyncError;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class DispatchDeskController extends Controller
{
    /**
     * Quote Desk — Loads at pricing/quote stage.
     * Push quote-ready loads to STLOADS, mark board-eligible vs internal-only.
     */
    public function quoteDesk()
    {
        $user = Auth::user();
        $roleId = $user->roles->first()?->id;

        // Loads at new/created stage (status_id 1) - pricing/quote review phase
        $query = LoadLeg::with(['load_master.stloadsHandoff', 'status_master'])
            ->whereIn('status_id', [1]); // New/Created

        if ($roleId != 1) { // Non-admin sees own loads
            $query->whereHas('load_master', fn($q) => $q->where('user_id', $user->id));
        }

        $legs = $query->orderByDesc('created_at')->paginate(20);

        $stloadsStats = [
            'eligible'   => $this->countEligibleForPush($legs),
            'published'  => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_PUBLISHED),
            'queued'     => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_QUEUED),
        ];

        return view('desk.quote', compact('legs', 'stloadsStats'));
    }

    /**
     * Tender Desk — Loads at tender/booking stage.
     * Push for external capacity, see board-exposure status, prevent duplicates.
     */
    public function tenderDesk()
    {
        $user = Auth::user();
        $roleId = $user->roles->first()?->id;

        // Loads at booked/pending stage (status_id 4) - tendering phase
        $query = LoadLeg::with(['load_master.stloadsHandoff', 'status_master', 'carrier'])
            ->whereIn('status_id', [1, 4]); // New + Booked

        if ($roleId != 1) {
            $query->whereHas('load_master', fn($q) => $q->where('user_id', $user->id));
        }

        $legs = $query->orderByDesc('created_at')->paginate(20);

        $stloadsStats = [
            'published'    => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_PUBLISHED),
            'push_failed'  => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_PUSH_FAILED),
            'withdrawn'    => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_WITHDRAWN),
        ];

        return view('desk.tender', compact('legs', 'stloadsStats'));
    }

    /**
     * Facility Desk — Loads at pickup-ready stage.
     * Confirm readiness (appointments, dock) before STLOADS exposure.
     */
    public function facilityDesk()
    {
        $user = Auth::user();
        $roleId = $user->roles->first()?->id;

        // Loads at booked/pickup stages (status_id 4, 5, 6) - facility readiness
        $query = LoadLeg::with(['load_master.stloadsHandoff', 'status_master', 'carrier'])
            ->whereIn('status_id', [4, 5, 6]); // Booked, Pickup Started, Arrived at Pickup

        if ($roleId != 1) {
            $query->whereHas('load_master', fn($q) => $q->where('user_id', $user->id));
        }

        $legs = $query->orderByDesc('created_at')->paginate(20);

        $stloadsStats = [
            'published'  => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_PUBLISHED),
            'no_handoff' => $this->countWithoutHandoff($legs),
        ];

        return view('desk.facility', compact('legs', 'stloadsStats'));
    }

    /**
     * Closeout Desk — Loads at delivery/complete stage.
     * See if STLOADS representation needs closing, reconciling, or archiving.
     */
    public function closeoutDesk()
    {
        $user = Auth::user();
        $roleId = $user->roles->first()?->id;

        // Loads at delivered/complete stages (status_id 9, 10) - closeout
        $query = LoadLeg::with(['load_master.stloadsHandoff', 'status_master'])
            ->whereIn('status_id', [9, 10]); // Arrived at Delivery, Completed

        if ($roleId != 1) {
            $query->whereHas('load_master', fn($q) => $q->where('user_id', $user->id));
        }

        $legs = $query->orderByDesc('created_at')->paginate(20);

        // Count loads that are delivered internally but still live on STLOADS
        $stillLive = 0;
        foreach ($legs as $leg) {
            $ho = $leg->load_master?->stloadsHandoff;
            if ($ho && in_array($ho->status, [StloadsHandoff::STATUS_PUBLISHED, StloadsHandoff::STATUS_QUEUED])) {
                $stillLive++;
            }
        }

        $stloadsStats = [
            'still_live' => $stillLive,
            'closed'     => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_CLOSED),
            'withdrawn'  => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_WITHDRAWN),
        ];

        return view('desk.closeout', compact('legs', 'stloadsStats'));
    }

    /**
     * Collections Desk — Loads at payout/finance stage.
     * See if STLOADS needs archiving downstream.
     */
    public function collectionsDesk()
    {
        $user = Auth::user();
        $roleId = $user->roles->first()?->id;

        // Loads at completed/paid stages (status_id 10, 11) - collections/finance
        $query = LoadLeg::with(['load_master.stloadsHandoff', 'status_master'])
            ->whereIn('status_id', [10, 11]); // Completed, Paid Out

        if ($roleId != 1) {
            $query->whereHas('load_master', fn($q) => $q->where('user_id', $user->id));
        }

        $legs = $query->orderByDesc('created_at')->paginate(20);

        // Count STLOADS representations not yet archived
        $needsArchive = 0;
        foreach ($legs as $leg) {
            $ho = $leg->load_master?->stloadsHandoff;
            if ($ho && !in_array($ho->status, [StloadsHandoff::STATUS_CLOSED, StloadsHandoff::STATUS_WITHDRAWN])) {
                $needsArchive++;
            }
        }

        $syncErrors = StloadsSyncError::unresolved()
            ->where('error_class', 'delivered_still_open')
            ->count();

        $stloadsStats = [
            'needs_archive' => $needsArchive,
            'sync_errors'   => $syncErrors,
            'closed'        => $this->countByHandoffStatus($legs, StloadsHandoff::STATUS_CLOSED),
        ];

        return view('desk.collections', compact('legs', 'stloadsStats'));
    }

    // ── Helpers ──────────────────────────────────────

    private function countEligibleForPush($legs): int
    {
        $count = 0;
        foreach ($legs as $leg) {
            if (!$leg->load_master?->stloadsHandoff) {
                $count++;
            }
        }
        return $count;
    }

    private function countByHandoffStatus($legs, string $status): int
    {
        $count = 0;
        foreach ($legs as $leg) {
            if ($leg->load_master?->stloadsHandoff?->status === $status) {
                $count++;
            }
        }
        return $count;
    }

    private function countWithoutHandoff($legs): int
    {
        $count = 0;
        foreach ($legs as $leg) {
            if (!$leg->load_master?->stloadsHandoff) {
                $count++;
            }
        }
        return $count;
    }
}
