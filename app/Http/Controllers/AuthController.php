<?php

namespace App\Http\Controllers;


use Illuminate\Http\Request;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\DB;
use App\Models\User;
use Spatie\Permission\Models\Role;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Mail;
use App\Mail\OtpMail;
use App\Models\KycDocuments;
use App\Models\UserDetail;
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
                $user = Auth::user();
                if ($user && $user->roles->first()?->id === 1) {
                    return redirect()->route('user_approval');
                } else {
                    return redirect()->route('manage-loads');
                }
                // return redirect()->route('dashboard');
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
                if ($user->status != 1 && $user->status != 4) {
                    $role = $user->roles()->first();
                    $logsController->createLog(__METHOD__, 'error', 'Login denied: User not approved', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                    return view('user_login_denial', compact('user', 'role'));
                } elseif ($user->status == 4) {
                    $logsController->createLog(__METHOD__, 'error', 'Login denied: User not verified', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                    return redirect()->route('otp', ['email' => $request->email])->with('error', 'Please verify your email first.');
                }
                Auth::login($user); // manually log in
                $request->session()->regenerate();

                $logsController->createLog(__METHOD__, 'success', 'Login Successful', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                if ($user->roles->first()?->id === 1) {
                    return redirect()->route('user_approval')->with('success', 'Login successful');
                } else {
                    return redirect()->route('manage-loads')->with('success', 'Login successful');
                }
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
            return view('enter_otp', [
                'to' => $request->email,
                'error' => 'Invalid or expired OTP.'
            ]);
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

    public function verifyOTPPassword(Request $request)
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
            return view('enter_otp_forget', [
                'to' => $request->email,
                'error' => 'Invalid or expired OTP.'
            ]);
        }

        // OTP is valid
        $user->update([
            'otp' => null,
            'otp_expires_at' => null,
        ]);

        return redirect()->route('new-password', $user->id)->with('success', 'Email verified successfully!');
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

            DB::beginTransaction();

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
                'otp_resend_count' => 1,
                'last_otp_resend_at' => Carbon::now(),
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

            DB::commit();

            // Prepare email content
            $fromAddress = config('mail.from.address');
            $fromName = config('mail.from.name');
            $to = $request->email;
            $subject = 'Your OTP Code';
            $body = "Your OTP for registration is: {$otp}\nIt will expire in 5 minutes.";

            Mail::raw($body, function ($message) use ($to, $subject, $fromAddress, $fromName) {
                $message->from($fromAddress, $fromName)
                    ->to($to)
                    ->subject($subject);
            });

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
            // \Log::error("Error sending OTP to {$request->email}: " . $e->getMessage());

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

    public function otp(LogsController $logsController, Request $request)
    {
        try {
            $referer = $request->headers->get('referer');

            $email = $request->email;
            if ($referer && str_contains($referer, 'forget-password')) {
                $user = User::where('email', $email)->first();
                $user->roles()->where('id', $request->id)->exists() ? $user : null;
            } else {
                $user = User::where('email', $email)->first();
            }

            if (!$user) {
                return redirect()->back()->withErrors(['error' => 'User not found. Please register first']);
            }
            $now = now();
            if (
                $user->last_otp_resend_at &&
                Carbon::parse($user->last_otp_resend_at)->diffInHours($now) < 1
            ) {
                // if ($user->last_otp_resend_at && $user->last_otp_resend_at->diffInHours($now) < 1) {
                if ($user->otp_resend_count >= 5) {
                    return redirect()->back()->withErrors(['error' => 'OTP resend limit reached. Try again after an hour.']);
                }
            } else {
                $user->otp_resend_count = 0;
            }

            $otp = rand(100000, 999999);
            $user->otp = $otp;
            $user->otp_expires_at = now()->addMinutes(5);
            $user->otp_resend_count += 1;
            $user->last_otp_resend_at = $now;
            $user->save();

            $fromAddress = config('mail.from.address');
            $fromName = config('mail.from.name');
            $to = $request->email;
            $subject = 'Your OTP Code';
            if ($referer && str_contains($referer, 'forget-password')) {
                $body = "Your OTP for forget password is: {$otp}\nIt will expire in 5 minutes.";
            } else {
                $body = "Your OTP for registration is: {$otp}\nIt will expire in 5 minutes.";
            }
            Mail::raw($body, function ($message) use ($to, $subject, $fromAddress, $fromName) {
                $message->from($fromAddress, $fromName)
                    ->to($to)
                    ->subject($subject);
            });
            if ($referer && str_contains($referer, 'forget-password')) {
                $logsController->createLog(__METHOD__, 'success', "OTP {$otp} sent to {$to} for password reset", null, json_encode(['email' => $to, 'otp' => $otp, 'from' => $fromAddress]));
                return view('auth.enter_otp_forget', compact('to'));
            } else {
                $logsController->createLog(__METHOD__, 'success', "OTP {$otp} sent to {$to} for password reset", null, json_encode(['email' => $to, 'otp' => $otp, 'from' => $fromAddress]));
                return view('auth.enter_otp', compact('to'));
            }
        } catch (\Exception $e) {
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    public function resendOtp(Request $request)
    {
        $email = $request->email;
        $user = User::where('email', $email)->first();

        if (!$user) {
            return response()->json(['success' => false, 'message' => 'User not found.']);
        }
        $now = now();
        // if ($user->last_otp_resend_at && $user->last_otp_resend_at->diffInHours($now) < 1) {
        if (
            $user->last_otp_resend_at &&
            Carbon::parse($user->last_otp_resend_at)->diffInHours($now) < 1
        ) {
            if ($user->otp_resend_count >= 5) {
                return response()->json([
                    'success' => false,
                    'message' => 'OTP resend limit reached. Try again after an hour.'
                ]);
            }
        } else {
            // Reset counter if it's been more than 1 hour
            $user->otp_resend_count = 0;
        }

        $otp = rand(100000, 999999);
        $user->otp = $otp;
        $user->otp_expires_at = now()->addMinutes(5);
        $user->otp_resend_count += 1;
        $user->last_otp_resend_at = $now;
        $user->save();

        $fromAddress = config('mail.from.address');
        $fromName = config('mail.from.name');
        $to = $request->email;
        $subject = 'Your OTP Code';
        $body = "Your OTP is: {$otp}\nIt will expire in 5 minutes.";
        Mail::raw($body, function ($message) use ($to, $subject, $fromAddress, $fromName) {
            $message->from($fromAddress, $fromName)
                ->to($to)
                ->subject($subject);
        });

        return response()->json(['success' => true]);
    }


    public function forgetPassword(LogsController $logsController, $id)
    {
        try {
            $role = Role::find($id);
            return view('auth.forget_password', compact('role'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    public function newPassword(LogsController $logsController, User $user)
    {
        try {
            return view('auth.new_password', compact('user'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
    }

    public function newPasswordPost(User $user, Request $request, LogsController $logsController)
    {
        try {
            $request->validate([
                'password' => 'required|string|min:6|confirmed',
            ]);

            $user->update([
                'password' => Hash::make($request->password),
            ]);
            // Custom application log
            $logsController->createLog(
                __METHOD__,
                'success',
                "New password set for user {$user->email}",
                null,
                json_encode(['user' => $user])
            );

            return redirect()->route('login', ['id' => $user->roles()->first()->id])
                ->with('success', 'Password updated successfully. Please login.');
        } catch (\Exception $e) {
            $logsController->createLog(
                __METHOD__,
                'error',
                'Eroor Setting New Password: ' . $e->getMessage(),
                null,
                json_encode(['user' => $user ?? 'N/A'])
            );

            return redirect()->back()->withErrors(['error' => 'Something went wrong: ' . $e->getMessage()]);
        }
    }

    public function onboardingForm(User $user)
    {
        $role = $user->roles()->first();
        // Return the onboarding form view
        return view('users.onboarding_form', compact('role', 'user'));
    }
    public function onboardingFormSave(User $user, Request $request)
    {
        $role = $user->roles()->first();

        // Validate common fields
        $request->validate([
            'company_name' => 'required|string|max:255',
            'company_address' => 'required|string|max:255',
            'cnic_front' => 'nullable|file|mimes:jpeg,jpg,png,pdf|max:2048',
            'cnic_back' => 'nullable|file|mimes:jpeg,jpg,png,pdf|max:2048',
        ]);

        $roleId = $role->id;
        $detailsData = [
            'company_name' => $request->company_name,
            'company_address' => $request->company_address,
        ];

        $documents = [];

        switch ($roleId) {
            case 2: // Carrier
                $detailsData['dot_number'] = $request->dot_number;
                $detailsData['mc_number'] = $request->mc_number;
                $detailsData['equipment_types'] = $request->equipment_types;

                $documents['certificate_of_insurance_carrier'] = $request->file('certificate_of_insurance_carrier');
                $documents['driver_roster'] = $request->file('driver_roster');
                $documents['safety_scorecard'] = $request->file('safety_scorecard');
                break;

            case 3: // Shipper
                $detailsData['business_entity_id'] = $request->business_entity_id;
                $detailsData['facility_address'] = $request->facility_address;
                $detailsData['fulfillment_contact_info'] = $request->fulfillment_contact_info;

                $documents['general_liability_insurance'] = $request->file('general_liability_insurance');
                break;

            case 4: // Broker
                $detailsData['fmcsa_broker_license_no'] = $request->fmcsa_broker_license_no;
                $detailsData['mc_authority_number'] = $request->mc_authority_number;

                $documents['bonding_proof_document'] = $request->file('bonding_proof_document');
                $documents['performance_history'] = $request->file('performance_history');
                break;

            case 5: // Forwarder
                $detailsData['freight_forwarder_license'] = $request->freight_forwarder_license;
                $detailsData['customs_license'] = $request->customs_license;

                $documents['certificate_of_insurance_freight_forwarder'] = $request->file('certificate_of_insurance_freight_forwarder');
                $documents['international_docs'] = $request->file('international_docs');
                $documents['port_authority_registration'] = $request->file('port_authority_registration');
                break;
        }

        if ($request->hasFile('cnic_front')) {
            $documents['cnic_front'] = $request->file('cnic_front');
        }

        if ($request->hasFile('cnic_back')) {
            $documents['cnic_back'] = $request->file('cnic_back');
        }

        // Save user_details (create or update)
        $userDetail = $user->details ?: new UserDetail();
        $userDetail->fill($detailsData);
        $userDetail->user_id = $user->id;
        $userDetail->save();

        // Save uploaded documents
        foreach ($documents as $key => $file) {
            if ($file) {
                $path = $file->store("kyc_documents/{$user->id}", 'public');

                KycDocuments::create([
                    'user_id' => $user->id,
                    'document_type' => $key,
                    'file_path' => $path,
                ]);
            }
        }

        // Only update the user's status
        $user->status = 3;
        $user->save();

        return redirect()->route('role')->with('success', 'Onboarding submitted. Awaiting admin approval.');
    }
}
