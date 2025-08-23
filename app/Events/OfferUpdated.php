<?php

namespace App\Events;

use App\Models\Offers;
use App\Models\Conversation;
use Illuminate\Broadcasting\PrivateChannel;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Queue\SerializesModels;

class OfferUpdated implements ShouldBroadcastNow
{
    use SerializesModels;

    public $offer;

    public function __construct(Offers $offer)
    {
        $this->offer = $offer->load('carrier', 'loadLeg');
    }

    public function broadcastOn()
    {
        // Find ALL conversation ids linked to this load leg
        $ids = Conversation::query()
            ->where('load_leg_id', $this->offer->load_leg_id)
            ->pluck('id')
            ->all();

        // Return an array of channels: convo.{conversation_id}
        return array_map(fn($id) => new PrivateChannel('convo.' . $id), $ids);
    }

    public function broadcastAs()
    {
        return 'offer.updated';
    }

    // Optional: trim payload
    public function broadcastWith()
    {
        return [
            'offer' => [
                'id'          => $this->offer->id,
                'load_leg_id' => $this->offer->load_leg_id,
                'carrier_id'  => $this->offer->carrier_id,
                'amount'      => (float) $this->offer->amount,
                'shipper_id' => $this->offer->loadLeg?->load_master?->user_id,
                'status_id'      => (int) $this->offer->status_id,
                'conversation_id' => $this->offer->conversation_id,
                'created_at' => $this->offer->created_at?->toIso8601String(),
                'updated_at' => $this->offer->updated_at?->toIso8601String(),
            ],
        ];
    }
}
