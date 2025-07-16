<?php

use Illuminate\Support\Facades\Route;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\DashboardController;
use App\Http\Controllers\AdminController;

/* ───────────────  GUEST‑ONLY ROUTES  ─────────────── */
Route::middleware('guest')->group(function () {
    // Login
    Route::get('/',            [AuthController::class, 'login'])->name('login');
    Route::post('/login',      [AuthController::class, 'verify'])->name('login.post');

    Route::get('/login', [AuthController::class, 'login'])->name('login');
    Route::post('/login', [AuthController::class, 'verify'])->name('login.post');

    Route::get('/register', [AuthController::class, 'registerForm'])->name('register.form');
    Route::post('/register', [AuthController::class, 'sendOtp'])->name('register'); // Keep commented for OTP flow

    Route::get('/forget-password', [AuthController::class, 'forgetPassword'])->name('forget-password');
    Route::get('/new-password', [AuthController::class, 'newPassword'])->name('new-password');

    Route::get('/otp', [AuthController::class, 'otp'])->name('otp');
    Route::post('/send-otp', [AuthController::class, 'sendOtp'])->name('otp.send');
    Route::post('/verify-otp', [AuthController::class, 'verifyOtp'])->name('otp.verify');
});

    Route::get('/', [AuthController::class, 'role'])->name('role');

    // 🔒 Auth-only routes
    Route::middleware('auth')->group(function () {
    
    Route::get('/dashboard', [DashboardController::class, 'dashboard'])->name('dashboard');
    Route::get('/user_approval', [AdminController::class, 'userApproval'])->name('user_approval');
    Route::post('/logout',   [AuthController::class, 'logout'])->name('logout');
});
