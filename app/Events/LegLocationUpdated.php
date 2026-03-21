<?php

namespace App\Events;

use App\Models\LoadLegLocation;
use Illuminate\Broadcasting\Channel;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Contracts\Broadcasting\ShouldBroadcast;
use Illuminate\Queue\SerializesModels;

class LegLocationUpdated implements ShouldBroadcast
{
    use InteractsWithSockets, SerializesModels;

    public $location;

    public function __construct(LoadLegLocation $location)
    {
        $this->location = $location;
    }

    public function broadcastOn()
    {
        return new Channel('leg.' . $this->location->leg_id . '.tracking');
    }

    public function broadcastAs()
    {
        return 'LegLocationUpdated';
    }

    public function broadcastWith()
    {
        return [
            'lat' => (float)$this->location->lat,
            'lng' => (float)$this->location->lng,
            'recorded_at' => $this->location->recorded_at->toDateTimeString()
        ];
    }
}
