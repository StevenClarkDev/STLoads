<?php

namespace App\Http\Controllers;


use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;
use App\Models\User;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Mail;

class AuthController extends Controller
{  public function landingPage()
    {
        return view('welcome'); // create this blade file if not done
    }
    public function login(LogsController $logsController)
    {
        if (auth()->check()) {
            return redirect()->route('dashboard');
        }

        return view('auth.login');
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
                $request->session()->regenerate();

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
    public function logout(Request $request)
    {
        Auth::logout(); // 💥 Logs out the user
        $request->session()->invalidate(); // 🧹 Clears session
        $request->session()->regenerateToken(); // 🔐 Prevent CSRF reuse

        return redirect()->route('login')->with('success', 'You have been logged out.');
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
    public function verifyOtp(Request $request, LogsController $logsController)
    {
        $request->validate([
            'otp' => 'required|digits:6',
        ]);

        if ($request->otp == session('register_otp')) {
            $data = session('pending_user');

            User::create($data);
            session()->forget(['pending_user', 'register_otp']);

            $logsController->createLog(__METHOD__, 'success', 'OTP matched and user registered', null, null);

            return redirect()->route('login')->with('success', 'Account created successfully.');
        } else {
            return back()->withErrors(['otp' => 'Invalid OTP. Please try again.']);
        }
    }
    public function sendOtp(Request $request)
    {
        $request->validate([
            'name' => 'required|string|max:255',
            'email' => 'required|string|email|unique:users,email',
            'password' => 'required|string|min:6|confirmed',
        ]);

        $otp = rand(100000, 999999);

        // Store user data + OTP in session
        session([
            'pending_user' => [
                'name' => $request->name,
                'email' => $request->email,
                'password' => Hash::make($request->password),
            ],
            'register_otp' => $otp,
        ]);

        // Send OTP via email
        Mail::raw("Your OTP is: $otp", function ($message) use ($request) {
            $message->to($request->email)
                ->subject('Your OTP for Registration');
        });

        return view('auth.enter-otp'); // Blade file where user inputs OTP
    }
}
