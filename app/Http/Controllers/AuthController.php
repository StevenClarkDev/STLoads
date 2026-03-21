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
use App\Models\ShipperDetail;
use Validator;
use Illuminate\Validation\ValidationException;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Str;
use Illuminate\Validation\Rule;


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
        $id = request()->query('id');

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
                    return redirect()->route('admin_dashboard');

                } else {
                    return redirect()->route('dashboard');
                }
                // return redirect()->route('dashboard');
            }
            $count_broker = User::role('Broker', 'web')->count();
            $count_shipper = User::role('Shipper', 'web')->count();
            $count_carrier = User::role('Carrier', 'web')->count();
            $count_freight_forwarder = User::role('Freight Forwarder', 'web')->count();
            return view('role', compact('count_broker', 'count_shipper', 'count_carrier', 'count_freight_forwarder'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request. ' . $e->getMessage()]);
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
                    $remarks = $user->latestHistory?->remarks ?? 'No remarks provided.';
                    $logsController->createLog(__METHOD__, 'error', 'Login denied: User not approved', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                    return view('user_login_denial', compact('user', 'role', 'remarks'));
                } elseif ($user->status == 4) {
                    $logsController->createLog(__METHOD__, 'error', 'Login denied: User not verified', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                    return redirect()->route('otp', ['email' => $request->email])->with('error', 'Please verify your email first.');
                }
                Auth::login($user); // manually log in
                $request->session()->regenerate();

                $logsController->createLog(__METHOD__, 'success', 'Login Successful', null, json_encode(['email' => $request->email, 'role_id' => $request->id]));
                if ($user->roles->first()?->id === 1) {
                    return redirect()->route('admin_dashboard')->with('success', 'Login successful');
                } else {
                    return redirect()->route('dashboard')->with('success', 'Login successful');
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

    public function userProfile(User $user, LogsController $logsController)
    {
        try {
            $role = $user->roles()->first();
            $logsController->createLog(__METHOD__, 'success', 'user is viewing ' . $user . ' profile', null, null);
            return view('user_profile', compact('user', 'role'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }

    public function profile(User $user, LogsController $logsController)
    {
        try {
            $role = $user->roles()->first();
            $logsController->createLog(__METHOD__, 'success', 'user is viewing ' . $user . ' profile', null, null);
            return view('users.user_profile', compact('user', 'role'));
        } catch (\Exception $e) {
            // Handle the exception, log it, or return an error response
            $logsController->createLog(__METHOD__, 'error', 'Failed to create log entry: ' . $e->getMessage(), null, null);
            return redirect()->back()->withErrors(['error' => 'An error occurred while processing your request.']);
        }
        // Create a log entry for the login attempt

    }

    public function save(User $user, Request $request)
    {
        // Basic validation
        $request->validate([
            'doc_id' => 'array',
            'doc_id.*' => [
                'nullable',
                'integer',
                Rule::exists('kyc_documents', 'id')->where(fn($q) => $q->where('user_id', $user->id)),
            ],
            'doc_name' => 'required|array|min:1',
            'doc_name.*' => 'required|string|max:255',
            'doc_type' => 'required|array',
            'doc_type.*' => ['required', Rule::in(['standard', 'blockchain'])],
            'documents' => 'array',
            'documents.*' => 'nullable|file|mimes:jpeg,jpg,png,pdf,docx'
                . '|mimetypes:image/jpeg,image/png,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document'
                . '|max:20480',
        ]);

        $ids = $request->input('doc_id', []);
        $names = $request->input('doc_name', []);
        $types = $request->input('doc_type', []);
        $files = $request->file('documents', []);

        // Require file for NEW rows (where doc_id is empty)
        foreach ($names as $i => $name) {
            if (empty($ids[$i]) && empty($files[$i])) {
                throw ValidationException::withMessages([
                    "documents.$i" => "File is required for new document row #" . ($i + 1) . ".",
                ]);
            }
        }

        DB::transaction(function () use ($user, $ids, $names, $types, $files) {
            $keepIds = [];

            foreach ($names as $i => $name) {
                $docId = $ids[$i] ?? null;
                $type = strtolower($types[$i] ?? 'standard');
                $file = $files[$i] ?? null;

                if ($docId) {
                    // Update existing
                    $doc = KycDocuments::where('user_id', $user->id)->where('id', $docId)->firstOrFail();
                } else {
                    // Create new
                    $doc = new KycDocuments(['user_id' => $user->id]);
                }

                // If a new file is uploaded, replace file + metadata
                if ($file) {
                    if ($doc->exists && $doc->file_path) {
                        Storage::disk('public')->delete($doc->file_path);
                    }
                    $path = $file->store("kyc_documents/{$user->id}", 'public');

                    $doc->file_path = $path;
                    $doc->original_name = $file->getClientOriginalName();
                    $doc->mime_type = $file->getClientMimeType();
                    $doc->file_size = $file->getSize();
                }

                // Always update name/type
                $doc->document_name = trim($name);
                $doc->document_type = $type;

                // Blockchain handling
                if ($type === 'blockchain') {
                    // Need a file_path (either existing or just uploaded)
                    if (!$doc->file_path) {
                        throw ValidationException::withMessages([
                            "documents.$i" => "A file is required to anchor blockchain document row #" . ($i + 1) . ".",
                        ]);
                    }
                    $abs = Storage::disk('public')->path($doc->file_path);
                    $hash = hash_file('sha256', $abs);

                    $doc->hash = $hash;
                    $doc->hash_algorithm = 'sha256';

                    // Create a new mock tx when there is a new upload or no tx yet
                    if ($file || empty($doc->mock_blockchain_tx)) {
                        $doc->mock_blockchain_tx = (string) Str::uuid();
                        $doc->mock_blockchain_timestamp = now();
                    }
                } else {
                    // Clear blockchain fields when switching to standard
                    $doc->hash = null;
                    $doc->hash_algorithm = null;
                    $doc->mock_blockchain_tx = null;
                    $doc->mock_blockchain_timestamp = null;
                }

                $doc->save();
                $keepIds[] = $doc->id;
            }

            // Delete documents removed in the UI (i.e., not submitted)
            KycDocuments::where('user_id', $user->id)
                ->whereNotIn('id', $keepIds)
                ->delete();

            // Set status back to "Awaiting admin approval"
            $user->status = 3;
            $user->save();
        });

        return back()->with('success', 'Profile updated. We’ll review your changes shortly.');
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
                return redirect()->route('admin_dashboard')->with('success', 'Login successful');
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
        $id = request()->query('id');
        $role_name = Role::find($id)->name ?? 'User';

        // if($id == 2){
        //     return view('auth.shipper_register', compact('id', 'role_name')); // create this blade file if not done
        // }else{
        return view('auth.register', compact('id', 'role_name')); // create this blade file if not done
        // }

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
                'ssn' => 'required|digits:9',
                'address' => 'required|string|max:255',
            ]);


            User::create([
                'name' => $request->name,
                'email' => $request->email,
                'password' => Hash::make($request->password),
            ]);

            $logsController->createLog(__METHOD__, 'success', 'User Registered', null, null);

            return redirect()->route('normal-login')->with('success', 'Account created successfully. Please login.');
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
                'ssn_no' => 'required',
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
                'ssn_no' => $request->ssn_no,
                'phone_no' => $request->phone_no,
                'ucr_hcc_no' => $request->ucr_hcc_no,
                'mc_cbsa_usdot_no' => $request->mc_cbsa_usdot_no,
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
                $path = $file->storeAs('uploads/profile_images', $filename, 'public'); // stores in storage/app/public/uploads/ssn_files

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
                if (!$user || !$user->roles()->where('id', $request->id)->exists()) {
                    return back()->withErrors(['error' => 'User not found for the selected role.']);
                }
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

            return redirect()->route('normal-login', ['id' => $user->roles()->first()->id])
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
        if ($role->id == 2) {
            return view('users.shipper_onboarding_form', compact('role', 'user'));
        } else {
            return view('users.onboarding_form', compact('role', 'user'));
        }
    }
    public function onboardingFormSave(User $user, Request $request)
    {
        $role = $user->roles()->first();
        if (!$role) {
            abort(422, 'User role not found.');
        }

        // Validate common fields + dynamic docs
        $request->validate([
            'company_name' => 'required|string|max:255',
            'company_address' => 'required|string|max:255',

            'doc_name' => 'required|array|min:1',
            'doc_name.*' => 'required|string|max:255',
            'doc_type' => 'required|array',
            'doc_type.*' => 'required|in:standard,blockchain',
            'documents' => 'required|array|min:1',
            'documents.*' => 'required|file|mimes:jpeg,jpg,png,pdf,docx|mimetypes:image/jpeg,image/png,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document|max:20480',
        ]);

        if (
            count($request->input('doc_name', [])) !== count($request->input('doc_type', [])) ||
            count($request->input('doc_name', [])) !== count($request->file('documents', []))
        ) {
            throw ValidationException::withMessages([
                'documents' => 'Document name, type, and file count must match.',
            ]);
        }

        $roleId = $role->id;
        $detailsData = [
            'company_name' => $request->company_name,
            'company_address' => $request->company_address,
        ];

        switch ($roleId) {
            case 2: // Carrier
                $detailsData['dot_number'] = $request->dot_number;
                $detailsData['mc_number'] = $request->mc_number;
                $detailsData['equipment_types'] = $request->equipment_types;
                break;
            case 3: // Shipper
                $detailsData['business_entity_id'] = $request->business_entity_id;
                $detailsData['facility_address'] = $request->facility_address;
                $detailsData['fulfillment_contact_info'] = $request->fulfillment_contact_info;
                break;
            case 4: // Broker
                $detailsData['fmcsa_broker_license_no'] = $request->fmcsa_broker_license_no;
                $detailsData['mc_authority_number'] = $request->mc_authority_number;
                break;
            case 5: // Forwarder
                $detailsData['freight_forwarder_license'] = $request->freight_forwarder_license;
                $detailsData['customs_license'] = $request->customs_license;
                break;
        }

        DB::transaction(function () use ($user, $detailsData, $request) {
            // Save user_details (create or update)
            $userDetail = $user->details ?: new UserDetail();
            $userDetail->fill($detailsData);
            $userDetail->user_id = $user->id;
            $userDetail->save();

            // Save KYC docs
            $names = $request->input('doc_name', []);
            $types = $request->input('doc_type', []);
            $files = $request->file('documents', []);

            foreach ($names as $i => $name) {
                $file = $files[$i];
                $storedPath = $file->store("kyc_documents/{$user->id}", 'public');

                $documentType = strtolower($types[$i] ?? 'standard');

                $payload = [
                    'user_id' => $user->id,
                    'document_name' => trim($name),
                    'document_type' => $documentType,
                    'file_path' => $storedPath,
                    'original_name' => $file->getClientOriginalName(),
                    'mime_type' => $file->getClientMimeType(),
                    'file_size' => $file->getSize(),
                ];

                // Mock blockchain anchoring for blockchain docs
                if ($documentType === 'blockchain') {
                    $absPath = Storage::disk('public')->path($storedPath);
                    $hash = hash_file('sha256', $absPath);

                    $payload['hash'] = $hash;
                    $payload['hash_algorithm'] = 'sha256';
                    $payload['mock_blockchain_tx'] = (string) Str::uuid(); // fake tx id
                    $payload['mock_blockchain_timestamp'] = now();
                }

                KycDocuments::create($payload);
            }

            // Only update the user's status
            $user->status = 3;
            $user->save();
        });

        return redirect()->route('role')->with('success', 'Onboarding submitted. Awaiting admin approval.');
    }

    public function onboardingFormSaveForShipper(User $user, Request $request)
    {
        // --- Validate Shipper fields + dynamic docs (no SSN) ---
        $request->validate([
            // common
            'company_name' => 'required|string|max:255',
            'company_address' => 'required|string|max:255',

            // shipper-specific
            'business_type' => 'required|string|max:255',
            'website' => 'nullable|url',
            'shipments_per_week' => 'required|numeric',
            'volume_or_weight_per_shipment' => 'required|string|max:255',
            'types_of_goods_being_shipped' => 'required|array',
            'packaging_type' => 'required|array',
            'types_of_delivery_services_needed' => 'required|array',
            'preferred_pickup_days' => 'required|array',
            'preferred_pickup_from_time' => 'required',
            'preferred_pickup_to_time' => 'required',
            'logistics_provider' => 'nullable|string',
            'preferred_payment_method' => 'required|array',
            'billing_contact_name' => 'required|string|max:255',
            'billing_email_address' => 'required|email|max:255',
            'tax_id' => 'required|string|max:255',
            'invoice_frequency' => 'required|string',
            'shipment_tracking' => 'required|string',
            'pickup_materials_supplied' => 'required|string',
            'demo_or_onboarding_call' => 'required|string',
            'preferred_communication_method' => 'required|array',
            'special_notes' => 'nullable|string|max:1000',
            'other_goods' => 'nullable|string|max:255',
            'other_payment' => 'nullable|string|max:255',

            // dynamic docs (same as onboardingFormSave)
            'doc_name' => 'required|array|min:1',
            'doc_name.*' => 'required|string|max:255',
            'doc_type' => 'required|array',
            'doc_type.*' => ['required', Rule::in(['standard', 'blockchain'])],
            'documents' => 'required|array|min:1',
            'documents.*' => 'required|file|mimes:jpeg,jpg,png,pdf,docx'
                . '|mimetypes:image/jpeg,image/png,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document'
                . '|max:20480',
        ]);

        if (
            count($request->input('doc_name', [])) !== count($request->input('doc_type', [])) ||
            count($request->input('doc_name', [])) !== count($request->file('documents', []))
        ) {
            throw ValidationException::withMessages([
                'documents' => 'Document name, type, and file count must match.',
            ]);
        }

        $detailsData = [
            'company_name' => $request->company_name,
            'company_address' => $request->company_address,
        ];

        DB::transaction(function () use ($user, $detailsData, $request) {
            // --- Save/Update UserDetail ---
            $userDetail = $user->details ?: new UserDetail();
            $userDetail->fill($detailsData);
            $userDetail->user_id = $user->id;
            $userDetail->save();

            // --- Save ShipperDetail (create or update) ---
            // If you have relation: $user->shipperDetail()
            $shipper = ShipperDetail::firstOrNew(['user_id' => $user->id]);
            $shipper->company_name = $request->company_name;
            $shipper->company_address = $request->company_address;
            $shipper->business_type = $request->business_type;
            $shipper->website = $request->website;
            $shipper->shipments_per_week = $request->shipments_per_week;
            $shipper->volume_or_weight_per_shipment = $request->volume_or_weight_per_shipment;
            $shipper->types_of_goods_being_shipped = json_encode($request->types_of_goods_being_shipped);
            $shipper->packaging_type = json_encode($request->packaging_type);
            $shipper->types_of_delivery_services_needed = json_encode($request->types_of_delivery_services_needed);
            $shipper->preferred_pickup_days = json_encode($request->preferred_pickup_days);
            $shipper->preferred_pickup_from_time = $request->preferred_pickup_from_time;
            $shipper->preferred_pickup_to_time = $request->preferred_pickup_to_time;
            $shipper->logistics_provider = $request->logistics_provider;
            $shipper->preferred_payment_method = json_encode($request->preferred_payment_method);
            $shipper->billing_contact_name = $request->billing_contact_name;
            $shipper->billing_email_address = $request->billing_email_address;
            $shipper->tax_id = $request->tax_id;
            $shipper->invoice_frequency = $request->invoice_frequency;
            $shipper->shipment_tracking = $request->shipment_tracking;
            $shipper->pickup_materials_supplied = $request->pickup_materials_supplied;
            $shipper->demo_or_onboarding_call = $request->demo_or_onboarding_call;
            $shipper->preferred_communication_method = json_encode($request->preferred_communication_method);
            $shipper->special_notes = $request->special_notes;
            $shipper->other_goods = $request->other_goods;
            $shipper->other_payment = $request->other_payment;
            $shipper->user_id = $user->id;
            $shipper->save();

            // --- Save dynamic KYC documents (with metadata + mock blockchain) ---
            $names = $request->input('doc_name', []);
            $types = $request->input('doc_type', []);
            $files = $request->file('documents', []);

            foreach ($names as $i => $name) {
                $file = $files[$i];
                $storedPath = $file->store("kyc_documents/{$user->id}", 'public');

                $documentType = strtolower($types[$i] ?? 'standard');

                $payload = [
                    'user_id' => $user->id,
                    'document_name' => trim($name),
                    'document_type' => $documentType,
                    'file_path' => $storedPath,
                    'original_name' => $file->getClientOriginalName(),
                    'mime_type' => $file->getClientMimeType(),
                    'file_size' => $file->getSize(),
                ];

                if ($documentType === 'blockchain') {
                    $absPath = Storage::disk('public')->path($storedPath);
                    $hash = hash_file('sha256', $absPath);

                    $payload['hash'] = $hash;
                    $payload['hash_algorithm'] = 'sha256';
                    $payload['mock_blockchain_tx'] = (string) Str::uuid();
                    $payload['mock_blockchain_timestamp'] = now();
                }

                KycDocuments::create($payload);
            }

            // --- Set status back to "Awaiting admin approval" ---
            $user->status = 3;
            $user->save();
        });

        return redirect()->route('role')->with('success', 'Onboarding submitted. Awaiting admin approval.');
    }
}
