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

// ── TMS Inbound API (service-to-service, Sanctum token auth) ──
Route::middleware('auth:sanctum')->prefix('stloads')->group(function () {
    Route::post('/push',    [\App\Http\Controllers\Api\TmsInboundController::class, 'push']);
    Route::post('/queue',   [\App\Http\Controllers\Api\TmsInboundController::class, 'queue']);
    Route::post('/requeue', [\App\Http\Controllers\Api\TmsInboundController::class, 'requeue']);
});