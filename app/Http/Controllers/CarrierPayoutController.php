<?php

namespace App\Http\Controllers;

use App\Services\StripeService;
use App\Models\User;
use Illuminate\Http\Request;

class CarrierPayoutController extends Controller
{
    public function __construct(private StripeService $stripe)
    {
    }

    public function redirectToOnboarding($id)
    {
        $user = User::findOrFail($id);
        $role = $user->roles()->first();
        abort_unless($role->id === 3, 403, 'Only carriers can enable payouts');

        // Create Express account if it doesn’t exist
        if (!$user->stripe_connect_account_id) {
            $acct = $this->stripe->client->accounts->create([
                'type' => 'express',
                'capabilities' => [
                    'transfers' => ['requested' => true],
                ],
                'email' => $user->email,
            ]);
            $user->stripe_connect_account_id = $acct->id;
            $user->save();
        }

         // Build return/refresh URLs. Use named routes that exist in your app.
        $refreshUrl = url()->temporarySignedRoute(
            'carrier.connect', now()->addMinutes(30), ['id' => $user->id]
        );

        // Where Stripe sends them back after finishing onboarding
        $returnUrl = route('normal-login', ['id' => $role->id]); // or your dashboard/home

        // Create the Stripe-hosted onboarding link
        $link = $this->stripe->client->accountLinks->create([
            'account' => $user->stripe_connect_account_id,
            'type' => 'account_onboarding',
            'refresh_url' => $refreshUrl,
            'return_url'  => $returnUrl,
        ]);

        // Generate account onboarding link
        // $link = $this->stripe->client->accountLinks->create([
        //     'account' => $user->stripe_connect_account_id,
        //     'type' => 'account_onboarding',
        //     'refresh_url' => route('carrier.connect', $user->id),
        //     'return_url' => route('normal-login', ['id' => $role->id]),
        // ]);

        return redirect()->away($link->url);
    }


    public function createOrLink(Request $req)
    {
        $user = $req->user();
        abort_unless($user->role === 'carrier', 403, 'Only carriers onboard payouts');

        // 1) Create Express connected account if not exists
        if (!$user->stripe_connect_account_id) {
            $acct = $this->stripe->client->accounts->create([
                'type' => 'express',
                'capabilities' => [
                    'transfers' => ['requested' => true],
                ],
                // Optionally pass country, email, business_type, etc.
            ]);
            $user->stripe_connect_account_id = $acct->id;
            $user->save();
        }

        // 2) Create account onboarding link
        $link = $this->stripe->client->accountLinks->create([
            'account' => $user->stripe_connect_account_id,
            'type' => 'account_onboarding',
            'refresh_url' => config('app.url') . '/carrier/connect/refresh',
            'return_url' => config('app.url') . '/settings/payouts?done=1',
        ]);

        return response()->json([
            'onboarding_url' => $link->url,
            'account_id' => $user->stripe_connect_account_id,
        ]);
    }

    public function refreshLink(Request $req)
    {
        $user = $req->user();
        abort_unless($user->stripe_connect_account_id, 404);
        $link = $this->stripe->client->accountLinks->create([
            'account' => $user->stripe_connect_account_id,
            'type' => 'account_onboarding',
            'refresh_url' => config('app.url') . '/carrier/connect/refresh',
            'return_url' => config('app.url') . '/settings/payouts?done=1',
        ]);
        return redirect()->away($link->url);
    }
}
