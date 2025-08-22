<?php

namespace App\Http\Controllers;

use App\Models\Offers;
use App\Models\LoadLeg;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;
use App\Events\OfferUpdated;


class OfferController extends Controller
{
    public function store(Request $req)
    {
        $data = $req->validate([
            'load_leg_id' => ['required', 'integer', 'exists:load_legs,id'],
            'amount'      => ['required', 'numeric', 'min:1'],
        ]);

        $carrierId = Auth::id();
        $loadLeg   = LoadLeg::query()->findOrFail($data['load_leg_id']);

        if ((int)$loadLeg->booked_carrier_id === $carrierId) {
            return response()->json(['message' => 'This load leg is already booked with you.'], 422);
        }
        if ((int)$loadLeg->status_id === 4) {
            return response()->json(['message' => 'This load leg is already booked.'], 422);
        }

        // (B) Enforce: carrier can’t offer again unless last one was declined
        $hasPending = Offers::query()
            ->where('load_leg_id', $loadLeg->id)
            ->where('carrier_id', $carrierId)
            ->where('status_id', 1)
            ->exists();

        if ($hasPending) {
            return response()->json(['message' => 'You already have a pending offer on this load. Wait for a decision.'], 422);
        }

        // Create as Pending
        $offer = Offers::create([
            'load_leg_id' => $loadLeg->id,
            'carrier_id'  => $carrierId,
            'amount'      => $data['amount'],
            'status_id'      => 1,
        ]);
        event(new OfferUpdated($offer));

        return response()->json([
            'message' => 'Offer submitted.',
            'offer'   => $offer,
        ]);
    }

    // Owner accepts an offer
    public function accept(Request $req, Offers $offer)
    {
        // Authorization stays before the transaction
        $loadLeg = $offer->loadLeg; // just to check existence for the auth line below
        abort_if(!$loadLeg, 404);
        abort_if((int)$loadLeg->load_master->user_id !== (int)Auth::id(), 403, 'Not allowed.');
        if ($offer->status_id !== 1) {
            return response()->json(['message' => 'Only pending offers can be accepted.'], 422);
        }

        // Do all writes + locking inside the transaction
        $updatedOffer = DB::transaction(function () use ($offer) {
            // Re-fetch the load leg in the transaction and lock it
            $loadLeg = $offer->loadLeg()->lockForUpdate()->firstOrFail();

            // 1) Accept this offer
            $offer->update(['status_id' => 3]);

            // 2) Decline other pending offers
            Offers::where('load_leg_id', $loadLeg->id)
                ->where('id', '<>', $offer->id)
                ->where('status_id', 1)
                ->update(['status_id' => 0]);

            // 3) Update load leg booking fields
            $loadLeg->update([
                'status_id'            => 4,
                'booked_carrier_id' => $offer->carrier_id,
                'booked_at'         => now(),
                'booked_amount'     => $offer->amount,
                'accepted_offer_id' => $offer->id,
            ]);

            return $offer->fresh(['carrier', 'loadLeg']);
        });

        // Broadcast AFTER commit
        event(new OfferUpdated($updatedOffer));

        return response()->json(['message' => 'Offer accepted and load booked.']);
    }

    public function decline(Request $req, Offers $offer)
    {
        $loadLeg = $offer->loadLeg;
        abort_if(!$loadLeg, 404);
        abort_if((int)$loadLeg->load_master->user_id !== (int)Auth::id(), 403, 'Not allowed.');

        if ($offer->status_id !== 1) {
            return response()->json(['message' => 'Offer is not pending.'], 422);
        }

        $offer->update(['status_id' => 0]);
        event(new OfferUpdated($offer));

        return response()->json(['message' => 'Offer declined.']);
    }
}
