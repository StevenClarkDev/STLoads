<?php

use Illuminate\Support\Facades\Route;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\DashboardController;
use App\Http\Controllers\RoleController;
use App\Http\Controllers\AdminController;
use App\Http\Controllers\UserController;
use App\Http\Controllers\LoadController;
use App\Http\Controllers\ChatController;
use App\Http\Controllers\LoadTypesController;
use App\Http\Controllers\EquipmentsController;
use App\Http\Controllers\CommodityTypesController;
use App\Http\Controllers\LocationsController;
use App\Http\Controllers\CitiesController;

/* ───────────────  GUEST‑ONLY ROUTES  ─────────────── */

Route::middleware('guest')->group(function () {
    // Login
    Route::get('/normal-login', [AuthController::class, 'login'])->name('normal-login');
    Route::get('/login', [AuthController::class, 'role'])->name('login');
    Route::get('/admin/login', [AuthController::class, 'adminLogin'])->name('admin.login');
    Route::post('/login', [AuthController::class, 'verify'])->name('login.post');
    Route::post('/admin/login', [AuthController::class, 'adminVerify'])->name('admin.login.post');

    Route::get('/register', [AuthController::class, 'registerForm'])->name('register.form');
    Route::post('/register', [AuthController::class, 'sendOtp'])->name('register');
    Route::post('/verify-otp', [AuthController::class, 'verifyOTP'])->name('verify-otp');
    Route::post('/verify-otp-forget', [AuthController::class, 'verifyOTPPassword'])->name('verify-otp-forget');
    Route::post('/otp/resend', [AuthController::class, 'resendOtp'])->name('otp.resend');


    Route::get('/forget-password/{id}', [AuthController::class, 'forgetPassword'])->name('forget-password');
    Route::get('/new-password/{user}', [AuthController::class, 'newPassword'])->name('new-password');
    Route::post('/new-password/{user}', [AuthController::class, 'newPasswordPost'])->name('new-password-post');

    Route::get('/otp', [AuthController::class, 'otp'])->name('otp');
    // Route::post('/send-otp', [AuthController::class, 'sendOtp'])->name('otp.send');
    // Route::post('/verify-otp', [AuthController::class, 'verifyOtp'])->name('otp.verify');
});

Route::get('/', [AuthController::class, 'role'])->name('role');
Route::get('/onboarding/{user}', [AuthController::class, 'onboardingForm'])->name('onboarding-form');
Route::post('/onboarding/{user}', [AuthController::class, 'onboardingFormSave'])->name('onboarding-form-save');
Route::post('/onboarding/shipper/{user}', [AuthController::class, 'onboardingFormSaveForShipper'])->name('onboarding-form-save-shipper');

// 🔒 Auth-only routes
Route::middleware('auth')->group(function () {

    Route::resource('roles', RoleController::class);
    Route::resource('users', UserController::class);
    Route::resource('load_types', LoadTypesController::class);
    Route::resource('equipments', EquipmentsController::class);
    Route::resource('commodity_types', CommodityTypesController::class);
    Route::resource('locations', LocationsController::class);
    Route::get('/dashboard', [DashboardController::class, 'dashboard'])->name('dashboard');
    Route::get('/user_approval', [AdminController::class, 'userApproval'])->name('user_approval');
    Route::get('/users_by_role/{id}', [UserController::class, 'usersByRole'])->name('users_by_role');
    Route::get('/user_profile/{user}', [AdminController::class, 'userProfile'])->name('user.profile');

    Route::post('/verify-admin-password', [AdminController::class, 'verifyPassword']);
    Route::get('/get-cnic-file/{id}', [AdminController::class, 'getCnicFiles']);
    Route::get('/get-user-file/{id}', [AdminController::class, 'getFiles']);
    Route::post('/approve-user', [UserController::class, 'approve']);
    Route::post('/reject-user', [UserController::class, 'reject']);

    // Load Management
    Route::get('/manage-loads', [LoadController::class, 'index'])->name('manage-loads');
    Route::post('/save_preferences', [LoadController::class, 'savePreferences'])->name('savePreferences');
    Route::get('/loads/add', [LoadController::class, 'add'])->name('loads.add');
    Route::post('/loads/add', [LoadController::class, 'store'])->name('loads.store');
    Route::get('/loads/edit/{load}', [LoadController::class, 'edit'])->name('loads.edit');
    Route::post('/loads/update/{load}', [LoadController::class, 'update'])->name('loads.update');
    Route::post('/loads/delete/{load}', [LoadController::class, 'delete'])->name('loads.delete');
    Route::get('/loads/view/{load}', [LoadController::class, 'view'])->name('loads.view');
    Route::get('/loads/bid/{load}', [LoadController::class, 'bid'])->name('loads.bid');

    // Chat
    Route::get('/chat/{load}', [ChatController::class, 'index'])->name('chat.load');
    Route::post('/chat/send', [ChatController::class, 'sendMessage'])->name('chat.send');
    Route::get('/chat/load-messages/{load}', [ChatController::class, 'loadMessages'])->name('chat.load-messages');
    Route::get('/chat', [ChatController::class, 'index'])->name('chat');

    Route::get('/api/countries/{country}/cities', [CitiesController::class, 'byCountry'])
    ->name('api.cities.by-country');

    Route::post('/logout',   [AuthController::class, 'logout'])->name('logout');
});
