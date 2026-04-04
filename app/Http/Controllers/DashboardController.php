<?php

namespace App\Http\Controllers;

use App\Models\Escrow;
use App\Models\KycDocuments;
use App\Models\LegDocuments;
use App\Models\LegEvent;
use App\Models\LoadDocuments;
use App\Models\LoadLeg;
use App\Models\Load;
use App\Models\StloadsHandoff;
use App\Models\StloadsSyncError;
use Carbon\Carbon;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class DashboardController extends Controller
{
    public function dashboard(LogsController $logsController)
    {
        try {
            $user = Auth::user();
            $role_id = $user->roles->first()?->id;

            // Default metrics initialized
            $metrics = $this->initializeMetrics();

            // Compute metrics based on user role
            if ($role_id == 3) {
                // For carrier
                $metrics = $this->computeCarrierMetrics($user, $metrics);
            } else {
                // For non-carrier (shipper/broker/etc.)
                $metrics = $this->computeNonCarrierMetrics($user, $metrics);
            }

            // Activity feed (Recent Activity)
            $activities = $this->getActivities($user, $role_id);

            // STLOADS handoff counts for dashboard widget
            $stloadsQueued    = StloadsHandoff::where('status', StloadsHandoff::STATUS_QUEUED)->count();
            $stloadsPublished = StloadsHandoff::where('status', StloadsHandoff::STATUS_PUBLISHED)->count();
            $stloadsFailed    = StloadsHandoff::whereIn('status', [StloadsHandoff::STATUS_PUSH_FAILED, StloadsHandoff::STATUS_REQUEUE_REQUIRED])->count();
            $stloadsWithdrawn = StloadsHandoff::where('status', StloadsHandoff::STATUS_WITHDRAWN)->count();
            $stloadsSyncErrors = StloadsSyncError::unresolved()->count();

            // Logs for dashboard loaded successfully
            $logsController->createLog(
                __METHOD__,
                'success',
                'Dashboard loaded',
                null,
                null
            );

            return view('dashboard', array_merge([
                'role_id' => $role_id,
                'activities' => $activities,
                'stloadsQueued' => $stloadsQueued,
                'stloadsPublished' => $stloadsPublished,
                'stloadsFailed' => $stloadsFailed,
                'stloadsWithdrawn' => $stloadsWithdrawn,
                'stloadsSyncErrors' => $stloadsSyncErrors,
            ], $metrics));

        } catch (\Exception $e) {
            
            $logsController->createLog(
                __METHOD__,
                'error',
                'Failed to load dashboard: ' . $e->getMessage(),
                null,
                null
            );

            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    private function initializeMetrics()
    {
        return [
            'completed_legs' => 0,
            'onTimeRate' => 0,
            'carrierRevenueMonth' => 0,
            'avgPayoutDays' => 0,
            'newRequests' => 0,
            'newRequestsToday' => 0,
            'onTimePickupRate' => 0,
            'onTimeDeliveryRate' => 0,
            'activeLegs' => 0,
            'monthlySpend' => 0
        ];
    }

    // Carrier specific metrics
    private function computeCarrierMetrics($user, $metrics)
    {
        // Completed legs
        $metrics['completed_legs'] = LoadLeg::where('status_id', 11)
            ->where('booked_carrier_id', $user->id)
            ->count();

        // On-time rate calculation
        $metrics['onTimePickupRate'] = $this->calculateOnTimeRate($user->id, 'pickup');
        $metrics['onTimeDeliveryRate'] = $this->calculateOnTimeRate($user->id, 'delivery');


        // Revenue for this month
        $metrics['carrierRevenueMonth'] = $this->calculateCarrierRevenue($user);

        // Average payout days
        $metrics['avgPayoutDays'] = $this->calculateAvgPayoutDays($user);

        // New requests for carrier
        $metrics['newRequests'] = LoadLeg::whereNull('booked_carrier_id')->count();
        $metrics['newRequestsToday'] = LoadLeg::whereNull('booked_carrier_id')->whereDate('created_at', today())->count();

        // Fetch documents for the user
        $metrics['documents'] = $this->fetchDocuments($user, 'carrier');

        // Active legs for this carrier (Today's legs)
        $metrics['loadLegs'] = LoadLeg::where('booked_carrier_id', $user->id)
            ->whereDate('pickup_date', today())
            ->orWhereDate('delivery_date', today())
            ->with('status_master')
            ->get();

        return $metrics;
    }

    // Non-carrier specific metrics
    private function computeNonCarrierMetrics($user, $metrics)
    {
        // New requests for non-carrier
        $metrics['newRequests'] = LoadLeg::whereHas('load_master', function ($q) use ($user) {
            $q->where('user_id', $user->id);
        })->count();

        $metrics['newRequestsToday'] = LoadLeg::whereHas('load_master', function ($q) use ($user) {
            $q->where('user_id', $user->id);
        })->whereDate('created_at', today())->count();

        // On-Time Pickup % and Delivery %
        $metrics['onTimePickupRate'] = $this->calculateOnTimeRate($user->id, 'pickup');
        $metrics['onTimeDeliveryRate'] = $this->calculateOnTimeRate($user->id, 'delivery');

        // Active Legs
        $metrics['activeLegs'] = LoadLeg::whereHas('load_master', function ($query) use ($user) {
            $query->where('user_id', $user->id);
        })->whereIn('status_id', [2, 3, 4, 5, 6, 7, 8, 9])->count();

        // Monthly Spend
        $metrics['monthlySpend'] = $this->calculateMonthlySpend($user);

        // Fetch documents for the user
        $metrics['documents'] = $this->fetchDocuments($user, 'non-carrier');

        // Active load legs for non-carrier
        $metrics['loadLegs'] = LoadLeg::whereHas('load_master', function ($query) use ($user) {
            $query->where('user_id', $user->id);
        })
            ->whereDate('pickup_date', today())
            ->orWhereDate('delivery_date', today())
            ->with('status_master')
            ->get();

        return $metrics;
    }

    // Fetch documents based on role type
    private function fetchDocuments($user, $roleType)
    {
        // Fetch KYC documents for the user
        $kycDocuments = KycDocuments::where('user_id', $user->id)
            ->latest()
            ->get();

        // Fetch documents based on role type
        $documents = collect();

        if ($roleType == 'carrier') {
            $legDocuments = LegDocuments::whereHas('load_leg', function ($query) use ($user) {
                $query->where('booked_carrier_id', $user->id);
            })
                ->latest()
                ->get();

            foreach ($legDocuments as $doc) {
                $documents->push((object) [
                    'type' => 'leg',
                    'name' => strtoupper(str_replace('_', ' ', $doc->type)) . '- OF ' . $doc->load_leg->leg_code,
                    'file_path' => $doc->path,
                    'uploaded_at' => $doc->created_at,
                ]);
            }
        } else {
            $loadDocuments = LoadDocuments::whereHas('load_master', function ($query) use ($user) {
                $query->where('user_id', $user->id);
            })
                ->latest()
                ->get();

            foreach ($loadDocuments as $doc) {
                $documents->push((object) [
                    'type' => 'leg',
                    'name' => $doc->document_name,
                    'file_path' => $doc->file_path,
                    'uploaded_at' => $doc->created_at,
                ]);
            }
        }

        // Add KYC Documents
        foreach ($kycDocuments as $doc) {
            $documents->push((object) [
                'type' => 'kyc',
                'name' => $doc->document_name,
                'file_path' => $doc->file_path,
                'uploaded_at' => $doc->created_at,
            ]);
        }

        // Sort by most recent upload
        return $documents->sortByDesc('uploaded_at')->values();
    }

    // Calculate on-time rate for a specific type (pickup or delivery)
    private function calculateOnTimeRate($userId, $type)
    {
        $legQuery = LoadLeg::whereHas('load_master', function ($query) use ($userId) {
            $query->where('user_id', $userId);
        });

        if ($type == 'pickup') {
            $completedQuery = $legQuery->whereNotNull('pickup_date')
                ->whereNotNull('pickup_arrived_at');
            $totalCompleted = $completedQuery->count();
            $onTimeCount = $completedQuery->whereColumn('pickup_arrived_at', '<=', 'pickup_date')->count();
        } else {
            $completedQuery = $legQuery->whereNotNull('delivery_date')
                ->whereNotNull('delivered_at');
            $totalCompleted = $completedQuery->count();
            $onTimeCount = $completedQuery->whereColumn('delivered_at', '<=', 'delivery_date')->count();
        }

        return $totalCompleted > 0 ? round(($onTimeCount / $totalCompleted) * 100) : 0;
    }

    // Calculate carrier's monthly revenue
    private function calculateCarrierRevenue($user)
    {
        return Escrow::where('payee_user_id', $user->id)
            ->where('status', 'released')
            ->whereMonth('created_at', now()->month)
            ->whereYear('created_at', now()->year)
            ->get()
            ->sum(function ($e) {
                return ($e->amount - $e->platform_fee) / 100; // Convert cents to dollars
            });
    }

    // Calculate average payout days for the carrier
    private function calculateAvgPayoutDays($user)
    {
        $payoutDurations = Escrow::where('status', 'released')
            ->where('payee_user_id', $user->id)
            ->with('leg')
            ->get()
            ->map(function ($e) {
                if (!$e->leg || !$e->leg->completed_at) {
                    return null;
                }
                return $e->updated_at->diffInDays($e->leg->completed_at);
            })
            ->filter(); // remove nulls

        return $payoutDurations->count() > 0 ? round($payoutDurations->avg()) : 0;
    }

    // Calculate monthly spend for non-carriers
    private function calculateMonthlySpend($user)
    {
        return Escrow::where('payer_user_id', $user->id)
            ->where('status', 'released')
            ->whereMonth('created_at', now()->month)
            ->whereYear('created_at', now()->year)
            ->get()
            ->sum(function ($e) {
                return ($e->amount - $e->platform_fee) / 100; // Convert cents to dollars
            });
    }

    // Fetch activity data (events, documents, etc.)
    private function getActivities($user, $role_id)
    {
        // Determine relevant legs based on role
        $legIds = ($role_id == 3) ?
            LoadLeg::where('booked_carrier_id', $user->id)->pluck('id') :
            LoadLeg::whereHas('load_master', function ($q) use ($user) {
                $q->where('user_id', $user->id);
            })->pluck('id');

        if ($legIds->isNotEmpty()) {
            $events = LegEvent::whereIn('leg_id', $legIds)->latest()->take(10)->get();
            $docs = LegDocuments::whereIn('leg_id', $legIds)->latest()->take(10)->get();
            $statusLegs = LoadLeg::whereIn('id', $legIds)->with('status_master')->latest()->take(10)->get();
        } else {
            $events = collect();
            $docs = collect();
            $statusLegs = collect();
        }

        // Merge activities into one collection
        return collect()
            ->merge($events->map(function ($e) {
                return (object) [
                    'activity_type' => 'event',
                    'leg_id' => $e->leg_id,
                    'type' => $e->type,
                    'created_at' => Carbon::parse($e->created_at),
                ];
            }))
            ->merge($docs->map(function ($d) {
                return (object) [
                    'activity_type' => 'document',
                    'leg_id' => $d->leg_id,
                    'type' => $d->type,
                    'created_at' => Carbon::parse($d->created_at),
                ];
            }))
            ->merge($statusLegs->map(function ($s) {
                return (object) [
                    'activity_type' => 'status',
                    'leg_id' => $s->id,
                    'type' => $s->status_master->name ?? 'updated',
                    'created_at' => Carbon::parse($s->updated_at),
                ];
            }))
            ->sortByDesc('created_at')->values();
    }
}
