<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Stripe\Webhook;
use App\Models\Escrow;
use App\Models\User;

class StripeWebhookController extends Controller
{
    public function handle(Request $req, LogsController $logs)
    {
        $payload = $req->getContent();
        $sigHeader = $req->header('Stripe-Signature');

        // Try both secrets (platform + connect)
        $secrets = array_filter([
            config('services.stripe.webhook_secret_platform'),
            config('services.stripe.webhook_secret_connect'),
        ]);

        if (empty($secrets)) {
            $logs->createLog(__METHOD__, 'error', 'No Stripe webhook secrets configured.', null, null);
            return response('No secret configured', 500);
        }

        // Verify signature with whichever secret matches
        $event = null;
        foreach ($secrets as $secret) {
            try {
                $event = Webhook::constructEvent($payload, $sigHeader, $secret);
                break; // success with this secret
            } catch (\Throwable $e) {
                // try next secret
            }
        }

        if (!$event) {
            $logs->createLog(__METHOD__, 'error', 'Stripe webhook signature verification failed.', null, null);
            return response('Invalid signature', 400);
        }

        try {
            switch ($event->type) {

                case 'payment_intent.succeeded': {
                    $pi = $event->data->object;
                    $escrow = Escrow::where('payment_intent_id', $pi->id)->first();
                    if ($escrow) {
                        $chargeId = $pi->latest_charge ?? null;

                        // Optional fallback, only if you really want:
                        if (!$chargeId && !empty($pi->charges->data[0])) {
                            $chargeId = $pi->charges->data[0]->id;
                        }
                        $escrow->update([
                            'status' => 'funded',
                            'charge_id' => $chargeId,
                        ]);
                    }
                    break;
                }

                case 'payment_intent.payment_failed': {
                    $pi = $event->data->object;
                    Escrow::where('payment_intent_id', $pi->id)->update(['status' => 'failed']);
                    break;
                }

                case 'account.updated': {
                    $acct = $event->data->object;

                    $payoutsEnabled = (bool) ($acct->payouts_enabled ?? false);
                    $kycPending = !empty($acct->requirements->currently_due);

                    User::where('stripe_connect_account_id', $acct->id)->update([
                        'payouts_enabled' => $payoutsEnabled,
                        'kyc_status' => $kycPending ? 'pending' : 'verified',
                        // Only activate when payouts are actually enabled
                        'status' => $payoutsEnabled ? 1 : 3,
                    ]);
                    break;
                }

                // Add more cases later: transfer.failed, charge.dispute.created, payout.failed, etc.
            }

        } catch (\Throwable $e) {
            $msg = 'Stripe webhook handler error for event '
                . ($event->type ?? 'unknown')
                . ': ' . $e->getMessage();
            $logs->createLog(__METHOD__, 'error', $msg, null, null);
            return response('Webhook handler error', 500);
        }

        // Always return 2xx to acknowledge receipt
        return response()->json(['ok' => true]);
    }
}
