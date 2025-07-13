<?php

use Illuminate\Support\Facades\Route;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\DashboardController;

// 👤 Guest-only routes (not logged in)
Route::middleware('guest')->group(function () {
    Route::get('/', [AuthController::class, 'landingPage'])->name('landingPage');
    Route::get('/login', [AuthController::class, 'login'])->name('login');
    Route::post('/login', [AuthController::class, 'verify'])->name('login.post');

    Route::get('/register', [AuthController::class, 'registerForm'])->name('register.form');
    // Route::post('/register', [AuthController::class, 'register'])->name('register'); // Keep commented for OTP flow

    Route::post('/send-otp', [AuthController::class, 'sendOtp'])->name('otp.send');
    Route::post('/verify-otp', [AuthController::class, 'verifyOtp'])->name('otp.verify');
});

// 🔒 Auth-only routes
Route::middleware('auth')->group(function () {
    Route::get('/dashboard', [DashboardController::class, 'dashboard'])->name('dashboard');
    Route::post('/logout', [AuthController::class, 'logout'])->name('logout');
});
