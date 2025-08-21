<?php

namespace App\Http\Controllers;

use App\Models\{Conversation, Load};
use App\Events\MessageSent;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;


class BidChatController extends Controller
{
    public function submit(Request $r, Load $load)
    {
        $this->authorize('createFromLoad', [\App\Models\Conversation::class, $load]);

        $data = $r->validate([
            'amount' => 'required|numeric|min:1|max:100000000',
            'note'   => 'nullable|string|max:2000',
        ]);

        $owner = $load->user; // load owner (shipper/broker/forwarder)
        $conversation = Conversation::firstOrCreate(
            ['load_id' => $load->id, 'carrier_id' => Auth::id()],
            ['shipper_id' => $owner->id]
        );

        $body = 'Offer: $' . number_format($data['amount'], 0) . (!empty($data['note']) ? ' — ' . $data['note'] : '');
        $msg = $conversation->messages()->create([
            'user_id' => Auth::id(),
            'body' => $body,
            'meta' => ['type' => 'bid', 'amount' => (float)$data['amount']],
        ]);

        broadcast(new MessageSent($msg))->toOthers();
        return redirect()->route('chat.room', $conversation);
    }
}
