<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;

class DashboardController extends Controller
{
    public function dashboard(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is attempting to log in', null, null);

            // Return the login view
            return view('dashboard');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt




    }
   
}
