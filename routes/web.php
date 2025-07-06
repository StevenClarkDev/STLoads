<?php

use Illuminate\Support\Facades\Route;
use App\Http\Controllers\AuthController;
use App\Http\Controllers\DashboardController;


Route::get('/', [AuthController::class, 'login'])->name('login');
Route::post('/login', [AuthController::class, 'verify'])->name('login.post');

Route::get('/dashboard', [DashboardController::class, 'dashboard'])->name('dashboard');



