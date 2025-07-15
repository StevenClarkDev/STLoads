<?php

use Illuminate\Support\Facades\Route;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\DashboardController;

/* ───────────────  GUEST‑ONLY ROUTES  ─────────────── */
Route::middleware('guest')->group(function () {
    // Login
    Route::get('/',            [AuthController::class, 'login'])->name('login');
    Route::post('/login',      [AuthController::class, 'verify'])->name('login.post');

    // Registration + OTP
    Route::get('/register',    [AuthController::class, 'registerForm'])->name('register.form');
    Route::post('/send-otp',   [AuthController::class, 'sendOtp'])->name('otp.send');
    Route::post('/verify-otp', [AuthController::class, 'verifyOtp'])->name('otp.verify');
});

/* ──────────────  AUTHENTICATED ROUTES  ───────────── */
Route::middleware('auth')->group(function () {
    Route::get('/dashboard', [DashboardController::class, 'dashboard'])->name('dashboard');
    Route::post('/logout',   [AuthController::class, 'logout'])->name('logout');
});
