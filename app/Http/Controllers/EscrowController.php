<?php

namespace App\Http\Controllers;

use App\Services\StripeService;
use Illuminate\Http\Request;
use App\Models\User;
use App\Models\Escrow;
use App\Models\Load;
use App\Models\LoadLeg;
use Illuminate\Support\Facades\Auth;

use Illuminate\Support\Str;

class EscrowController extends Controller
{
    public function __construct(private StripeService $stripe)
    {
    }

    public function fund(Request $req, LoadLeg $leg)
    {
        // AuthZ: only load creator (shipper/broker/forwarder) who owns this leg
        // $this->authorize('fund', $leg);
        $user = Auth::user();

        $carrier = User::findOrFail($leg->booked_carrier_id);
        abort_unless($carrier->stripe_connect_account_id, 409, 'Carrier has not completed payout setup');
        $amountInCents = (int) round(($leg->booked_amount > 0 ? $leg->booked_amount : $leg->amount) * 100);
        $platformFeeInCents = (int) round(($leg->platform_fee_minor ?? 0) * 100);
        $escrow = Escrow::firstOrCreate(
            ['leg_id' => $leg->id],
            [
                'payer_user_id' => $user->id,
                'payee_user_id' => $carrier->id,
                'currency' => 'usd',                   // e.g. 'usd'
                'amount' => $amountInCents,
                'platform_fee' => $platformFeeInCents,
                'status' => 'unfunded',
                'transfer_group' => 'LEG_' . $leg->id,
            ]
        );
        LoadLeg::where('id', $leg->id)->update([
            'status_id'=> 8,
        ]);

        // Create PaymentIntent
        $pi = $this->stripe->client->paymentIntents->create([
            'amount' => $escrow->amount,
            'currency' => $escrow->currency,
            'transfer_group' => $escrow->transfer_group,
            'automatic_payment_methods' => ['enabled' => true],
            'description' => "Funding leg {$leg->leg_code} for $" . number_format(($escrow->amount / 100), 2),
            'metadata' => ['leg_id' => (string) $leg->id],
        ]);

        $escrow->update(['payment_intent_id' => $pi->id]);

        return response()->json([
            'clientSecret' => $pi->client_secret,
            'paymentIntentId' => $pi->id,
        ]);
    }

    public function release(Request $request, Escrow $escrow)
    {
        // 2) Escrow must be funded
        if ($escrow->status !== 'funded') {
            abort(409, 'Escrow is not funded or already released.');
        }

        // 3) Make sure we have a charge and payee
        if (!$escrow->charge_id) {
            abort(409, 'No charge_id on escrow. Cannot transfer.');
        }

        $carrier = User::findOrFail($escrow->payee_user_id);

        if (!$carrier->stripe_connect_account_id) {
            abort(409, 'Carrier does not have a Stripe Connect account.');
        }

        // 4) Calculate payout amount (total - platform_fee)
        $payoutAmount = $escrow->amount - $escrow->platform_fee;
        if ($payoutAmount <= 0) {
            abort(409, 'Invalid payout amount calculated.');
        }

        // 5) Create transfer from platform → connected account
        //    Uses source_transaction to pull from that specific charge balance.
        $transfer = $this->stripe->client->transfers->create([
            'amount'           => $payoutAmount,
            'currency'         => $escrow->currency,
            'destination'      => $carrier->stripe_connect_account_id,
            'source_transaction' => $escrow->charge_id,
            'transfer_group'   => $escrow->transfer_group,
        ]);

        // 6) Save and mark as released
        $escrow->update([
            'status'      => 'released',   // or 'paid_out'
            'transfer_id' => $transfer->id,
        ]);
        $escrow->leg->update([
            'status_id'      => 11,   // or 'paid_out'
            'completed_at' => now(),
        ]);

        return back()->with('success', 'Payment released to carrier.');
    }
}
