<?php

namespace App\Http\Controllers;

use App\Models\KycDocuments;
use App\Models\User;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Hash;


class AdminController extends Controller
{
    public function userApproval(LogsController $logsController)
    {
        try {
            $users = User::where('status', 3)->get();
            $usersCount = User::where('status', 3)->count();
            $usersCountToday = User::where('status', 3)
                ->whereDate('created_at', \Carbon\Carbon::today())
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
                    \Carbon\Carbon::now()->startOfMonth(),
                    \Carbon\Carbon::now()->endOfMonth()
                ])->count();
            $totalUsersApprovedThisMonthPercentage = $totalUsersApproved > 0
                ? round(($totalUsersApprovedThisMonth / $totalUsersApproved) * 100, 2)
                : 0;
            $totalUsersRejected = User::where('status', 2)->count();
            $totalUsersRejectedThisMonth = User::where('status', 2)
                ->whereBetween('rejected_at', [
                    \Carbon\Carbon::now()->startOfMonth(),
                    \Carbon\Carbon::now()->endOfMonth()
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
