<?php

use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;

Route::get('/user', function (Request $request) {
    return $request->user();
})->middleware('auth:sanctum');

Route::middleware('auth:sanctum')->group(function () {
    Route::post('/carrier/connect', [\App\Http\Controllers\CarrierPayoutController::class, 'createOrLink']);
    Route::get('/carrier/connect/refresh', [\App\Http\Controllers\CarrierPayoutController::class, 'refreshLink']);
    Route::post('/legs/{leg}/escrow/release', [\App\Http\Controllers\EscrowController::class, 'release']); // after delivery
});