<?php

use Illuminate\Support\Facades\Route;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\DashboardController;
use App\Http\Controllers\RoleController;
use App\Http\Controllers\AdminController;
use App\Http\Controllers\UserController;

/* ───────────────  GUEST‑ONLY ROUTES  ─────────────── */

Route::middleware('guest')->group(function () {
    // Login
    Route::get('/login', [AuthController::class, 'login'])->name('login');
    Route::get('/admin/login', [AuthController::class, 'adminLogin'])->name('admin.login');
    Route::post('/login', [AuthController::class, 'verify'])->name('login.post');
    Route::post('/admin/login', [AuthController::class, 'adminVerify'])->name('admin.login.post');

    Route::get('/register', [AuthController::class, 'registerForm'])->name('register.form');
    Route::post('/register', [AuthController::class, 'sendOtp'])->name('register');
    Route::post('/verify-otp', [AuthController::class, 'verifyOTP'])->name('verify-otp');
    Route::post('/otp/resend', [AuthController::class, 'resendOtp'])->name('otp.resend');


    Route::get('/forget-password', [AuthController::class, 'forgetPassword'])->name('forget-password');
    Route::get('/new-password', [AuthController::class, 'newPassword'])->name('new-password');

    Route::get('/otp', [AuthController::class, 'otp'])->name('otp');
    // Route::post('/send-otp', [AuthController::class, 'sendOtp'])->name('otp.send');
    // Route::post('/verify-otp', [AuthController::class, 'verifyOtp'])->name('otp.verify');
});

Route::get('/', [AuthController::class, 'role'])->name('role');
Route::get('/onboarding/{user}', [AuthController::class, 'onboardingForm'])->name('onboarding-form');
Route::post('/onboarding/{user}', [AuthController::class, 'onboardingFormSave'])->name('onboarding-form-save');

// 🔒 Auth-only routes
Route::middleware('auth')->group(function () {

    Route::resource('roles', RoleController::class);
    Route::resource('users', UserController::class);
    Route::get('/dashboard', [DashboardController::class, 'dashboard'])->name('dashboard');
    Route::get('/user_approval', [AdminController::class, 'userApproval'])->name('user_approval');
    Route::get('/users_by_role/{id}', [UserController::class, 'usersByRole'])->name('users_by_role');
    Route::get('/user_profile/{user}', [AdminController::class, 'userProfile'])->name('user.profile');

    Route::post('/verify-admin-password', [AdminController::class, 'verifyPassword']);
    Route::get('/get-cnic-file/{id}', [AdminController::class, 'getCnicFiles']);
    Route::get('/get-user-file/{id}', [AdminController::class, 'getFiles']);
    Route::post('/approve-user', [UserController::class, 'approve']);
    Route::post('/reject-user', [UserController::class, 'reject']);

    Route::post('/logout',   [AuthController::class, 'logout'])->name('logout');
});
