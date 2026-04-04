<?php

use App\Http\Controllers\LoadLegController;
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
use App\Http\Controllers\StripeWebhookController;
use App\Http\Controllers\EscrowController;
use App\Http\Controllers\CarrierPayoutController;
use App\Http\Controllers\BidChatController;
use App\Http\Controllers\ConversationController;
use App\Http\Controllers\OfferController;
use App\Http\Controllers\StloadsOperationsController;
use App\Http\Controllers\DispatchDeskController;
use App\Events\TestPing;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Storage;
use App\Models\{Conversation};
use Illuminate\Foundation\Http\Middleware\VerifyCsrfToken;

/* ───────────────  GUEST‑ONLY ROUTES  ─────────────── */

Route::middleware('guest')->group(function () {
    // Login
    Route::get('/normal-login', [AuthController::class, 'login'])->name('normal-login');
    Route::get('/carrier/connect/{id}', [CarrierPayoutController::class, 'redirectToOnboarding'])->name('carrier.connect')->middleware('signed');
    Route::get('/login', [AuthController::class, 'role'])->name('login');
    Route::get('/admin/login', [AuthController::class, 'adminLogin'])->name('admin.login');
    Route::post('/login', [AuthController::class, 'verify'])->name('login.post');
    Route::get('/guest_profile/{user}', [AuthController::class, 'userProfile'])->name('guest.profile');
    Route::post('/profile/revise/{user}', [AuthController::class, 'save'])->name('profile.revise.save');
    Route::post('/admin/login', [AuthController::class, 'adminVerify'])->name('admin.login.post');

    Route::get('/register', [AuthController::class, 'registerForm'])->name('register.form');
    Route::post('/register', [AuthController::class, 'sendOtp'])->name('register');
    Route::post('/register/shipper', [AuthController::class, 'sendOtpShipper'])->name('register.shipper');
    Route::post('/register/carrier', [AuthController::class, 'sendOtpCarrier'])->name('register.carrier');
    Route::post('/register/broker', [AuthController::class, 'sendOtpBroker'])->name('register.broker');
    Route::post('/register/freight-forwarder', [AuthController::class, 'sendOtpFreightForwarder'])->name('register.freight-forwarder');
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
Route::post('/stripe/webhook', [StripeWebhookController::class, 'handle'])
    ->withoutMiddleware([VerifyCsrfToken::class])   // <-- disables CSRF just for this route
    ->name('stripe.webhook');


// 🔒 Auth-only routes
Route::middleware('auth')->group(function () {

    Route::resource('roles', RoleController::class);
    Route::resource('users', UserController::class);
    Route::resource('load_types', LoadTypesController::class);
    Route::resource('equipments', EquipmentsController::class);
    Route::resource('commodity_types', CommodityTypesController::class);
    Route::resource('locations', LocationsController::class);
    Route::get('/dashboard', [DashboardController::class, 'dashboard'])->name('dashboard');
    Route::get('/admin_dashboard', [AdminController::class, 'dashboard'])->name('admin_dashboard');
    Route::get('/user_approval', [AdminController::class, 'userApproval'])->name('user_approval');
    Route::get('/users_by_role/{id}', [UserController::class, 'usersByRole'])->name('users_by_role');
    Route::get('/user_profile/{user}', [AdminController::class, 'userProfile'])->name('user.profile');

    Route::post('/verify-admin-password', [AdminController::class, 'verifyPassword']);
    Route::get('/admin/change-password', [AdminController::class, 'changePassword'])->name('admin.change-password');
    Route::post('/admin/change-password', [AdminController::class, 'updatePassword'])->name('admin.change-password.update');
    Route::get('/get-ssn-file/{id}', [AdminController::class, 'getSsnFiles']);
    Route::get('/get-user-file/{id}', [AdminController::class, 'getFiles']);
    Route::get('/serve-kyc-file', [AdminController::class, 'serveKycFile'])->name('admin.serve-kyc-file');
    Route::post('/update-status/{id}', [UserController::class, 'updateStatus'])->name('user.update-status');

    // Load Management
    Route::get('/admin-manage-loads', [LoadController::class, 'adminIndex'])->name('admin.manage-loads');
    Route::post('/load-update-status/{id}', [LoadController::class, 'updateStatus'])->name('load.update-status');
    Route::get('/admin/loads/view/{load}', [LoadController::class, 'adminView'])->name('admin.loads.view');
    Route::get('/manage-loads', [LoadController::class, 'index'])->name('manage-loads');

    // STLOADS Operations
    // Dispatch Desk Routes
    Route::get('/desk/quote', [DispatchDeskController::class, 'quoteDesk'])->name('desk.quote');
    Route::get('/desk/tender', [DispatchDeskController::class, 'tenderDesk'])->name('desk.tender');
    Route::get('/desk/facility', [DispatchDeskController::class, 'facilityDesk'])->name('desk.facility');
    Route::get('/desk/closeout', [DispatchDeskController::class, 'closeoutDesk'])->name('desk.closeout');
    Route::get('/desk/collections', [DispatchDeskController::class, 'collectionsDesk'])->name('desk.collections');

    Route::get('/stloads/operations', [StloadsOperationsController::class, 'index'])->name('stloads.operations');
    Route::get('/stloads/handoff/{handoff}', [StloadsOperationsController::class, 'show'])->name('stloads.handoff.show');
    Route::get('/stloads/sync-errors', [StloadsOperationsController::class, 'syncErrors'])->name('stloads.sync-errors');
    Route::post('/stloads/sync-error/{error}/resolve', [StloadsOperationsController::class, 'resolveError'])->name('stloads.sync-error.resolve');
    Route::get('/stloads/reconciliation', [StloadsOperationsController::class, 'reconciliation'])->name('stloads.reconciliation');
    Route::post('/stloads/reconciliation/scan', [StloadsOperationsController::class, 'runScan'])->name('stloads.reconciliation.scan');
    Route::post('/stloads/handoff/{handoff}/force-sync', [StloadsOperationsController::class, 'forceSync'])->name('stloads.handoff.force-sync');
    Route::get('/profile/{user}', [AuthController::class, 'profile'])->name('profile');
    Route::get('/profile/{user}/edit', [AuthController::class, 'editProfile'])->name('profile.edit');
    Route::post('/profile/{user}/update', [AuthController::class, 'updateProfile'])->name('profile.update');
    Route::post('/save_preferences', [LoadController::class, 'savePreferences'])->name('savePreferences');
    Route::get('/loads/view/{load}', [LoadController::class, 'view'])->name('loads.view');
    Route::post('/load/revise/{load}', [LoadController::class, 'save'])->name('load.revise.save');
    Route::get('/loads/add', [LoadController::class, 'add'])->name('loads.add');
    Route::post('/loads/add', [LoadController::class, 'store'])->name('loads.store');
    Route::get('/loads/edit/{load}', [LoadController::class, 'edit'])->name('loads.edit');
    Route::post('/loads/update/{load}', [LoadController::class, 'update'])->name('loads.update');
    Route::post('/loads/delete/{load}', [LoadController::class, 'delete'])->name('loads.delete');
    Route::get('/loads/view/{load}', [LoadController::class, 'view'])->name('loads.view');
    Route::post('/load-legs/{load_leg}/book', [LoadController::class, 'book'])->name('load-legs.book');

    // Route::get('/loads/bid/{load}', [LoadController::class, 'bid'])->name('loads.bid');

    // Chat
    // Route::get('/chat/{load}', [ChatController::class, 'index'])->name('chat.load');
    // Route::post('/chat/send', [ChatController::class, 'sendMessage'])->name('chat.send');
    // Route::get('/chat/load-messages/{load}', [ChatController::class, 'loadMessages'])->name('chat.load-messages');
    // Route::get('/chat', [ChatController::class, 'index'])->name('chat');

    Route::post('/loads/{loadLeg}/bid', [BidChatController::class, 'submit'])->name('loads.bid');
    Route::get('/chat', function () {
        $userId = Auth::id();

        $conversations = Conversation::query()
            ->where(fn($q) => $q->where('carrier_id', $userId)->orWhere('shipper_id', $userId))
            ->latest('updated_at')
            ->get();

        if ($conversations->isNotEmpty()) {
            // Go to most recent convo
            return redirect()->route('chat.room', $conversations->first());
        }

        // No conversations yet → render empty state
        return view('chat.index', [
            'roomId' => null,
            'messages' => collect(),   // empty collection
            'conversation' => null,
            'conversations' => $conversations,
        ]);
    })->name('chat.index');
    Route::get('/chat/{conversation}', [ConversationController::class, 'show'])->name('chat.room');
    Route::post('/chat/{conversation}', [ConversationController::class, 'store']);

    Route::post('/offers', [OfferController::class, 'store'])->name('offers.store');
    Route::post('/offers/{offer}/accept', [OfferController::class, 'accept'])->name('offers.accept');
    Route::post('/offers/{offer}/decline', [OfferController::class, 'decline'])->name('offers.decline');

    Route::get('/api/countries/{country}/cities', [CitiesController::class, 'byCountry'])
        ->name('api.cities.by-country');

    Route::post('/legs/{leg}/escrow/fund', [EscrowController::class, 'fund'])->name('legs.escrow.fund');
    Route::post('/admin/escrows/{escrow}/release', [EscrowController::class, 'release'])->name('admin.escrows.release');

    Route::post('/legs/{leg}/pickup/start', [LoadLegController::class, 'startPickup'])->name('leg.pickup.start');
    Route::post('/legs/{leg}/location', [LoadLegController::class, 'storeLocation'])
        ->name('leg.location.store');
    Route::get('/legs/{leg}/track', [LoadLegController::class, 'track'])
        ->name('leg.track');

    Route::post('/legs/{leg}/pickup/arrived', [LoadLegController::class, 'arrivedPickup'])->name('leg.pickup.arrived');
    Route::post('/legs/{leg}/pickup/depart',  [LoadLegController::class, 'departPickup'])->name('leg.pickup.depart');
    Route::post('/legs/{leg}/delivery/arrived', [LoadLegController::class, 'arrivedDelivery'])->name('leg.delivery.arrived');
    Route::post('/legs/{leg}/delivery/complete', [LoadLegController::class, 'completeDelivery'])->name('leg.delivery.complete');

    Route::post('/legs/{leg}/documents', [LoadLegController::class, 'storeDocs'])->name('leg.documents.store');


    Route::post('/logout', [AuthController::class, 'logout'])->name('logout');
});
