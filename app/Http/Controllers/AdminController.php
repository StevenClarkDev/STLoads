<?php

namespace App\Http\Controllers;

use App\Models\Escrow;
use App\Models\KycDocuments;
use App\Models\LegDocuments;
use App\Models\LegEvent;
use App\Models\LoadLeg;
use App\Models\Logs;
use App\Models\User;
use Carbon\Carbon;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Hash;


class AdminController extends Controller
{
    public function dashboard(LogsController $logsController)
    {
        try {
            // Example data you might want on the dashboard (optional)
            $users = User::where('status', 3)->get();
            $usersCount = User::where('status', 3)->count();
            $usersCountToday = User::where('status', 3)
                ->whereDate('created_at', Carbon::today())
                ->count();
            $totalShipperApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 2);
                })
                ->count();
            $totalCarriersApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 3);
                })
                ->count();
            $totalBrookersApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 4);
                })
                ->count();
            $totalFreightForwardersApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 5);
                })
                ->count();
            $totalUsersApproved = User::where('status', 1)->count();
            $totalUsersApprovedThisMonth = User::where('status', 1)
                ->whereBetween('approved_at', [
                    Carbon::now()->startOfMonth(),
                    Carbon::now()->endOfMonth()
                ])->count();
            $totalUsersApprovedThisMonthPercentage = $totalUsersApproved > 0
                ? round(($totalUsersApprovedThisMonth / $totalUsersApproved) * 100, 2)
                : 0;
            $totalUsersRejected = User::where('status', 2)->count();
            $totalUsersRejectedThisMonth = User::where('status', 2)
                ->whereBetween('rejected_at', [
                    Carbon::now()->startOfMonth(),
                    Carbon::now()->endOfMonth()
                ])->count();
            $totalUsersRejectedThisMonthPercentage = $totalUsersRejected > 0
                ? round(($totalUsersRejectedThisMonth / $totalUsersRejected) * 100, 2)
                : 0;

            // ---------- Calculate Funds on Hold ----------
            $fundsOnHold = Escrow::where('status', 'funded')
                ->get()
                ->sum(function ($e) {
                    return ($e->amount - $e->platform_fee) / 100; // cents → dollars
                });


            // ---------- Calculate Approval SLAs ----------
            $totalApprovedThisMonth = User::where('status', 1)
                ->whereBetween('approved_at', [
                    Carbon::now()->startOfMonth(),
                    Carbon::now()->endOfMonth()
                ])
                ->count();

            $totalApproved = User::where('status', 1)->count();
            $approvalSLAPercentage = $totalApproved > 0
                ? round(($totalApprovedThisMonth / $totalApproved) * 100, 2)
                : 0;

            // ---------- Recent Activity (Dynamic) ----------
            $recentActivities = $this->getActivities();
            // Log action
            $logsController->createLog(__METHOD__, 'success', 'Admin accessed Dashboard', null, null);

            // Send data to the dashboard view (optional)
            return view('admin.admin_dashboard', compact('users', 'totalUsersApprovedThisMonthPercentage', 'totalUsersRejectedThisMonthPercentage', 'usersCount', 'usersCountToday', 'totalShipperApproved', 'totalCarriersApproved', 'totalBrookersApproved', 'totalFreightForwardersApproved', 'totalUsersApproved', 'totalUsersRejected', 'fundsOnHold', 'approvalSLAPercentage', 'recentActivities'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Dashboard load failed: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while loading the dashboard.']);
        }
    }

    private function getActivities()
    {
        // Fetch the latest 5 logs from the logs database for all users
        $logs = Logs::where('status', 'success')
            ->where('function', 'NOT LIKE', '%AdminController%') // Exclude logs with function containing 'AdminController'
            ->orderBy('log_date', 'desc')
            ->take(15)
            ->get();

        // Fetch recent events (from the LoadLegs, LegEvents, etc.)
        $events = LegEvent::latest()->take(5)->get(); // Fetch recent 5 events
        $docs = LegDocuments::latest()->take(5)->get(); // Fetch recent 5 documents
        $statusLegs = LoadLeg::latest()->take(5)->with('status_master')->get(); // Fetch recent 5 load legs

        // Merge all activities into one collection
        return collect()
            // Map logs to activity entries
            ->merge($logs->map(function ($log) {
                // Map the function or status to a specific activity type
                $icon = 'clock';
                $color = 'secondary';
                $text = '';
                if (stripos($log->function, 'approved') !== false) {
                    $icon = 'check-circle';
                    $color = 'success';
                    $text = "Approved: {$log->message}";
                } elseif (stripos($log->function, 'rejected') !== false) {
                    $icon = 'x-circle';
                    $color = 'danger';
                    $text = "Rejected: {$log->message}";
                } elseif (stripos($log->function, 'verified') !== false) {
                    $icon = 'user-check';
                    $color = 'primary';
                    $text = "Verified: {$log->message}";
                } elseif (stripos($log->function, 'recalculated') !== false) {
                    $icon = 'refresh-cw';
                    $color = 'warning';
                    $text = "Recalculated: {$log->message}";
                } elseif (stripos($log->status, 'delayed') !== false) {
                    $icon = 'alert-circle';
                    $color = 'danger';
                    $text = "Delayed: {$log->message}";
                } else {
                    $icon = 'info';
                    $color = 'secondary';
                    $text = "{$log->message}";
                }

                // Return log activity data
                return (object) [
                    'activity_type' => 'log',
                    'icon' => $icon,
                    'color' => $color,
                    'message' => $text,
                    'time' => Carbon::parse($log->log_date)->diffForHumans()
                ];
            }))
            // Map event activities
            ->merge($events->map(function ($e) {
                $icon = 'truck';
                $color = 'primary';
                $text = "Event: " . ucwords(str_replace('_', ' ', $e->type));

                return (object) [
                    'activity_type' => 'event',
                    'icon' => $icon,
                    'color' => $color,
                    'message' => $text,
                    'time' => Carbon::parse($e->created_at)->diffForHumans()
                ];
            }))
            // Map document activities
            ->merge($docs->map(function ($d) {
                $icon = 'file-text';
                $color = 'warning';
                $text = "Document: " . strtoupper($d->type);

                return (object) [
                    'activity_type' => 'document',
                    'icon' => $icon,
                    'color' => $color,
                    'message' => $text,
                    'time' => Carbon::parse($d->created_at)->diffForHumans()
                ];
            }))
            // Map load leg status updates
            ->merge($statusLegs->map(function ($s) {
                $icon = 'check-circle';
                $color = 'success';
                $text = "Status: {$s->status_master->name} (Leg #{$s->id})";

                return (object) [
                    'activity_type' => 'status',
                    'icon' => $icon,
                    'color' => $color,
                    'message' => $text,
                    'time' => Carbon::parse($s->updated_at)->diffForHumans()
                ];
            }))
            // Sort activities by most recent (created_at or updated_at)
            ->sortByDesc('time')->values();
    }


    public function userApproval(LogsController $logsController)
    {
        try {
            $users = User::where('status', 3)->get();
            $usersCount = User::where('status', 3)->count();
            $usersCountToday = User::where('status', 3)
                ->whereDate('created_at', Carbon::today())
                ->count();
            $totalShipperApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 2);
                })
                ->count();
            $totalCarriersApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 3);
                })
                ->count();
            $totalBrookersApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 4);
                })
                ->count();
            $totalFreightForwardersApproved = User::where('status', 1)
                ->whereHas('roles', function ($query) {
                    $query->where('id', 5);
                })
                ->count();
            $totalUsersApproved = User::where('status', 1)->count();
            $totalUsersApprovedThisMonth = User::where('status', 1)
                ->whereBetween('approved_at', [
                    Carbon::now()->startOfMonth(),
                    Carbon::now()->endOfMonth()
                ])->count();
            $totalUsersApprovedThisMonthPercentage = $totalUsersApproved > 0
                ? round(($totalUsersApprovedThisMonth / $totalUsersApproved) * 100, 2)
                : 0;
            $totalUsersRejected = User::where('status', 2)->count();
            $totalUsersRejectedThisMonth = User::where('status', 2)
                ->whereBetween('rejected_at', [
                    Carbon::now()->startOfMonth(),
                    Carbon::now()->endOfMonth()
                ])->count();
            $totalUsersRejectedThisMonthPercentage = $totalUsersRejected > 0
                ? round(($totalUsersRejectedThisMonth / $totalUsersRejected) * 100, 2)
                : 0;
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to Approve Users', null, null);
            return view('admin.user_approval', compact('users', 'totalUsersApprovedThisMonthPercentage', 'totalUsersRejectedThisMonthPercentage', 'usersCount', 'usersCountToday', 'totalShipperApproved', 'totalCarriersApproved', 'totalBrookersApproved', 'totalFreightForwardersApproved', 'totalUsersApproved', 'totalUsersRejected'));
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    public function carriers(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to Approve Users', null, null);
            $users = User::all();
            return view('admin.carriers', compact('users'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }
    public function shippers(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to Approve Users', null, null);
            $users = User::all();
            return view('admin.shippers', compact('users'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }
    public function brookers(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to Approve Users', null, null);
            $users = User::all();
            return view('admin.brookers', compact('users'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }
    public function freightForwarders(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to Approve Users', null, null);
            $users = User::all();
            return view('admin.freighters', compact('users'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }
    public function userProfile(User $user, LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'Admin is viewing ' . $user . ' profile', null, null);
            return view('admin.user_profile', compact('user'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }

    public function verifyPassword(Request $request)
    {
        if (Hash::check($request->password, Auth::user()->password)) {
            return response()->json(['success' => true]);
        }
        return response()->json(['success' => false]);
    }

    public function getCnicFiles($id)
    {
        $kycDocs = KycDocuments::where('user_id', $id)
            ->whereIn('document_type', ['cnic_front', 'cnic_back'])
            ->get();

        $files = [];

        foreach ($kycDocs as $doc) {
            $filePath = storage_path("app/public/{$doc->file_path}");

            if (file_exists($filePath)) {
                $files[] = [
                    'url' => asset("storage/{$doc->file_path}"),
                    'type' => $doc->document_type,
                ];
            }
        }

        return response()->json(['files' => $files]);
    }
    public function getFiles($id)
    {
        $kycDocs = KycDocuments::where('user_id', $id)
            ->get();

        $files = [];

        foreach ($kycDocs as $doc) {
            $filePath = storage_path("app/public/{$doc->file_path}");

            if (file_exists($filePath)) {
                $files[] = [
                    'url' => asset("storage/{$doc->file_path}"),
                    'type' => $doc->document_type,
                ];
            }
        }

        return response()->json(['files' => $files]);
    }
}
