<?php

namespace App\Http\Controllers;


use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;
use App\Models\User;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Mail;
use App\Mail\OtpMail; // Make sure this is at the top of your file

class AuthController extends Controller
{
    public function landingPage()
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
                'role' => 'required|string|max:255',
                'dob' => 'required|date',
                'gender' => 'required|string|in:Male,Female,Other',
                'cnic' => 'required|digits:13',
                'address' => 'required|string|max:255',
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

        if (!session()->has('register_otp') || !session()->has('pending_user')) {
            return redirect()->route('register.form')->withErrors(['error' => 'Session expired. Please register again.']);
        }

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
    
    public function sendOtp(Request $request, LogsController $logsController)
    {
        try {
            $request->validate([
                'name' => 'required|string|max:255',
                'email' => 'required|string|email|unique:users,email',
                'password' => 'required|string|min:6|confirmed',
                'role' => 'required|string|max:255',
                'dob' => 'required|date',
                'gender' => 'required|string|in:Male,Female,Other',
                'cnic' => 'required|digits:13',
                'address' => 'required|string|max:255',
            ]);


            $otp = rand(100000, 999999);

            // Log OTP generation
            \Log::info("Generated OTP: {$otp} for email: {$request->email}");

            session([
                'pending_user' => [
                    'name' => $request->name,
                    'email' => $request->email,
                    'password' => Hash::make($request->password),
                    'role' => $request->role,
                    'dob' => $request->dob,
                    'gender' => $request->gender,
                    'cnic' => $request->cnic,
                    'address' => $request->address,
                ],
                'register_otp' => $otp,
            ]);


            // Prepare email content
            $fromAddress = config('mail.from.address');
            $fromName = config('mail.from.name');
            $to = $request->email;
            $subject = 'Your OTP Code';
            $body = "Your OTP for registration is: {$otp}\nIt will expire in 5 minutes.";

            // Log full email structure
            \Log::info("Email Message Details", [
                'From' => "{$fromName} <{$fromAddress}>",
                'To' => $to,
                'Subject' => $subject,
                'Body' => $body,
            ]);

            // Send OTP via raw email
            Mail::raw($body, function ($message) use ($to, $subject, $fromAddress, $fromName) {
                $message->from($fromAddress, $fromName)
                    ->to($to)
                    ->subject($subject);
            });

            // Log success
            \Log::info("OTP email successfully sent to {$to}");

            // Custom application log
            $logsController->createLog(
                __METHOD__,
                'success',
                "OTP {$otp} sent to {$to}",
                null,
                json_encode(['email' => $to, 'otp' => $otp, 'from' => $fromAddress])
            );

            return view('auth.enter_otp');

        } catch (\Exception $e) {
            \Log::error("Error sending OTP to {$request->email}: " . $e->getMessage());

            $logsController->createLog(
                __METHOD__,
                'error',
                'OTP sending failed: ' . $e->getMessage(),
                null,
                json_encode(['email' => $request->email ?? 'N/A'])
            );

            return redirect()->back()->withErrors(['error' => 'Something went wrong: ' . $e->getMessage()]);
        }
    }

    public function otp(LogsController $logsController)
    {
        try {
            return view('auth.enter_otp');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    public function forgetPassword(LogsController $logsController)
    {
        try {
            return view('auth.forget_password');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    public function newPassword(LogsController $logsController)
    {
        try {
            return view('auth.new_password');
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

}
