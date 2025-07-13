<?php

namespace App\Http\Controllers;


use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;
use App\Models\User;

class AuthController extends Controller
{
    public function login(LogsController $logsController)
    {
        try {
            //$logsController->createLog(__METHOD__, 'success', 'User is attempting to log in', null, null);

            // Return the login view
            return view('auth.login');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt




    }

    public function role(LogsController $logsController)
    {
        try {
            return view('role');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }
    
    public function verify(Request $request, LogsController $logsController)
    {
        try {
            // dd(Hash::make('123456'));
            $request->validate([
                'email' => 'required|email',
                'password' => 'required|min:6',
            ]);
            if (auth()->attempt($request->only('email', 'password'))) {
                
                //dd('Login successful');
                $logsController->createLog(__METHOD__, 'success', 'Login SucessFull', null, null);
                return redirect()->route('dashboard')->with('success', 'Login successful');
            } else {
                $logsController->createLog(__METHOD__, 'error', 'Login denied: Invalid credentials', null, null);
                return redirect()->back()->withErrors(['error' => 'Invalid credentials']);
            }
        } catch (\Exception $e) {
            //dd($e);
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Login denied ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }
    public function registerForm()
    {
        return view('auth.register'); // create this blade file if not done
    }

    public function register(Request $request, LogsController $logsController)
    {
        try {
            $request->validate([
                'name' => 'required|string|max:255',
                'email' => 'required|string|email|unique:users,email',
                'password' => 'required|string|min:6|confirmed',
            ]);

            User::create([
                'name' => $request->name,
                'email' => $request->email,
                'password' => Hash::make($request->password),
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User Registered', null, null);

            return redirect()->route('login')->with('success', 'Account created successfully. Please login.');
        } catch (\Exception $e) {
            dd($e);
            $logsController->createLog(__METHOD__, 'error', 'Registration failed: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'Something went wrong during registration.']);
        }
    }
}
