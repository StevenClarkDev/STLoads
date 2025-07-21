<?php

namespace App\Http\Controllers;


use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;
use App\Models\User;
use Spatie\Permission\Models\Role;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Mail;
use App\Mail\OtpMail;
use Carbon\Carbon;


class AuthController extends Controller
{
    public function landingPage()
    {
        return view('welcome'); // create this blade file if not done
    }
    public function login(LogsController $logsController)
    {
        // if (auth()->check()) {
        //     return redirect()->route('dashboard');
        // }
        $id  = request()->query('id');

        return view('auth.login', compact('id'));
    }
    public function adminLogin(LogsController $logsController)
    {
        try {
            $logsController->createLog(__METHOD__, 'info', 'Admin login page accessed', null, null);
            return view('auth.admin_login');
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    public function role(LogsController $logsController)
    {
        try {
            if (Auth::check()) {
                return redirect()->route('dashboard');
            }
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
            $request->validate([
                'email' => 'required|email',
                'password' => 'required|min:6',
            ]);

            $user = User::where('email', $request->email)->first();

            if (!$user || !Hash::check($request->password, $user->password)) {
                $logsController->createLog(__METHOD__, 'error', 'Login denied: Invalid credentials', null, json_encode(['email' => $request->email]));
                return redirect()->back()->withErrors(['error' => 'Invalid credentials']);
            }

            // Check if user has the required role id
            if ($user->roles()->where('id', $request->id)->exists()) {
                if($user->status != 1) {
                    $role = $user->roles()->first();
                    $logsController->createLog(__METHOD__, 'error', 'Login denied: User not approved', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                    return view('user_login_denial', compact('user', 'role'));
                }
                Auth::login($user); // manually log in
                $request->session()->regenerate();

                $logsController->createLog(__METHOD__, 'success', 'Login Successful', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                return redirect()->route('user_approval')->with('success', 'Login successful');
            } else {
                $logsController->createLog(__METHOD__, 'error', 'Login denied: Role mismatch', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                return redirect()->back()->withErrors(['error' => 'Login denied: Role mismatch']);
            }
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Login denied: ' . $e->getMessage(), null, json_encode(['email' => $request->email ?? 'N/A']));
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }
    public function adminVerify(Request $request, LogsController $logsController)
    {
        try {
            $request->validate([
                'email' => 'required|email',
                'password' => 'required|min:6',
            ]);

            $user = User::where('email', $request->email)->first();

            if (!$user || !Hash::check($request->password, $user->password)) {
                $logsController->createLog(__METHOD__, 'error', 'Login denied: Invalid credentials', null, json_encode(['email' => $request->email]));
                return redirect()->back()->withErrors(['error' => 'Invalid credentials']);
            }

            // Check if user has the required role id
            if ($user->roles()->where('id', 1)->exists()) {
                Auth::login($user);
                $request->session()->regenerate();

                $logsController->createLog(__METHOD__, 'success', 'Login Successful', null, json_encode(['email' => $request->email, 'role_id' => 1]));
                return redirect()->route('user_approval')->with('success', 'Login successful');
            } else {
                $logsController->createLog(__METHOD__, 'error', 'Login denied: Role mismatch', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                return redirect()->back()->withErrors(['error' => 'Login denied: Role mismatch']);
            }
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Login denied: ' . $e->getMessage(), null, json_encode(['email' => $request->email ?? 'N/A']));
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }


    public function logout(Request $request)
    {
        Auth::logout(); // 💥 Logs out the user
        $request->session()->invalidate(); // 🧹 Clears session
        $request->session()->regenerateToken(); // 🔐 Prevent CSRF reuse

        return redirect()->route('role')->with('success', 'You have been logged out.');
    }
    public function registerForm()
    {
        $id  = request()->query('id');
        $role_name = Role::find($id)->name ?? 'User';

        return view('auth.register', compact('id', 'role_name')); // create this blade file if not done
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
            $logsController->createLog(__METHOD__, 'error', 'Registration failed: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'Something went wrong during registration.']);
        }
    }
    public function verifyOtp(Request $request)
    {
        $request->validate([
            'email' => 'required|email',
            'otp' => 'required|array|size:6',
            'otp.*' => 'required|digits:1',
        ]);

        // Combine the 6 OTP digits into a single string
        $otp = implode('', $request->otp);

        // Find the user with a matching email and unexpired OTP
        $user = User::where('email', $request->email)
            ->where('otp', $otp)
            ->where('otp_expires_at', '>', now())
            ->first();

        if (!$user) {
            return back()->withErrors(['otp' => 'Invalid or expired OTP.']);
        }

        // OTP is valid
        $user->update([
            'email_verified_at' => now(),
            'otp' => null,
            'otp_expires_at' => null,
            'status' => 0, // optional: mark active
        ]);

        // Auth::login($user); // Log the user in

        return redirect()->route('onboarding-form', $user->id)->with('success', 'Email verified successfully!');
    }



    public function sendOtp(Request $request, LogsController $logsController)
    {
        try {
            $request->validate([
                'name' => 'required|string|max:255',
                'email' => 'required|string|email|unique:users,email',
                'password' => 'required|string|min:6|confirmed',
                'role_id' => 'required',
                'dob' => 'required|date',
                'gender' => 'required|string',
                'cnic_no' => 'required',
                'address' => 'required|string|max:255',
            ]);


            $otp = rand(100000, 999999);
            $otpExpiry = Carbon::now()->addMinutes(5);

            // Log OTP generation
            // \Log::info("Generated OTP: {$otp} for email: {$request->email}");

            // session([
            //     'pending_user' => [
            //         'name' => $request->name,
            //         'email' => $request->email,
            //         'password' => Hash::make($request->password),
            //         'role' => $request->role,
            //         'dob' => $request->dob,
            //         'gender' => $request->gender,
            //         'cnic' => $request->cnic,
            //         'address' => $request->address,
            //     ],
            //     'register_otp' => $otp,
            // ]);

            $user = User::create([
                'name' => $request->name,
                'email' => $request->email,
                'password' => Hash::make($request->password),
                'dob' => $request->dob,
                'gender' => $request->gender,
                'cnic_no' => $request->cnic_no,
                'address' => $request->address,
                'otp' => $otp,
                'otp_expires_at' => $otpExpiry,
                'email_verified_at' => null,
                'status' => 4,
            ]);

            $role = Role::findOrFail($request->role_id);
            $user->assignRole($role->name);

            if ($request->hasFile('user_image')) {
                $file = $request->file('user_image');
                $filename = time() . '_' . $file->getClientOriginalName();
                $path = $file->storeAs('uploads/profile_images', $filename, 'public'); // stores in storage/app/public/uploads/cnic_files

                // Optional: save this path to the database
                $user->image = $path;
                $user->save();
            }


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

            return view('auth.enter_otp', compact('to'));
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

    public function onboardingForm(User $user)
    {
        $role = $user->roles()->first();
        // Return the onboarding form view
        return view('users.onboarding_form', compact('role', 'user'));
    }
    public function onboardingFormSave(User $user)
    {
        $role = $user->roles()->first();

        return view('users.onboarding_form', compact('role'));
    }
}
