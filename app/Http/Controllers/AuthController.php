<?php

namespace App\Http\Controllers;


use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;

class AuthController extends Controller
{
    public function login(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'success', 'User is attempting to log in', null, null);

            // Return the login view
            return view('auth.login');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt




    }
    public function verify(Request $request, LogsController $logsController)
    {
        try {
            // dd(Hash::make('123456'));
            $request->validate([
                'email' => 'required|email',
                'password' => 'required|min:6',
            ]);
            if(auth()->attempt($request->only('email', 'password'))) {
                $logsController->createLog(__METHOD__, 'success', 'Login SucessFull', null, null);
                return redirect()->route('dashboard')->with('success', 'Login successful');
            }
           
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Login denied ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        }
}
