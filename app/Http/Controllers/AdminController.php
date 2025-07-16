<?php

namespace App\Http\Controllers;

use App\Models\User;
use Illuminate\Http\Request;

class AdminController extends Controller
{
    public function userApproval(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to Approve Users', null, null);
            $users = User::all();
            return view('admin.user_approval', compact('users'));
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
            $logsController->createLog(__METHOD__, 'success', 'Admin is attempting to Approve Users', null, null);
            return view('admin.user_profile', compact('user'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }
   
}
