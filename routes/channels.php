<?php

use Illuminate\Support\Facades\Broadcast;
use App\Models\Conversation;
use App\Support\Roles;

/*
|--------------------------------------------------------------------------
| Broadcast Channels
|--------------------------------------------------------------------------
|
| Here you may register all of the event broadcasting channels that your
| application supports. The given channel authorization callbacks are
| used to check if an authenticated user can listen to the channel.
|
*/

// Example private channel for conversations
Broadcast::channel('convo.{conversationId}', function ($user, $conversationId) {
    $c = Conversation::find($conversationId);

    return $user->hasRole(Roles::ADMIN)
        || in_array($user->id, [$c->carrier_id, $c->shipper_id], true);

    // Allow admin, carrier, or shipper/owner of the load
    // if ($user->hasRole(Roles::ADMIN) || in_array($user->id, [$c->carrier_id, $c->shipper_id], true)) {
    //     return ['id' => $user->id, 'name' => $user->name];
    // }

    // return false;
});
