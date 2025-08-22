<?php

// routes/channels.php
use Illuminate\Support\Facades\Broadcast;
use App\Models\Conversation;
use App\Support\Roles;

Broadcast::channel('convo.{conversationId}', function ($user, $conversationId) {
    $c = Conversation::find($conversationId);
    if (! $c) return false;

    if ($user->hasRole(Roles::ADMIN)
        || in_array($user->id, [$c->carrier_id, $c->shipper_id], true)) {
        // return array => Laravel includes it as channel_data in JSON response
        return ['id' => $user->id, 'name' => $user->name];
    }
    return false;
});
