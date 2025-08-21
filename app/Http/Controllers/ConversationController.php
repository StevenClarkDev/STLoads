<?php

namespace App\Http\Controllers;

use App\Models\{Conversation};
use App\Events\MessageSent;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class ConversationController extends Controller
{
    public function show(Conversation $conversation)
    {
        // dd('here');
        $this->authorize('view', $conversation);

        $messages = $conversation->messages()
            ->with('user')
            ->latest()
            ->take(50)
            ->get()
            ->reverse();

        $userId = Auth::id(); // ✅ robust + IDE friendly

        $conversations = \App\Models\Conversation::query()
            ->where(fn($q) => $q->where('carrier_id', $userId)
                ->orWhere('shipper_id', $userId))
            ->latest('updated_at')
            ->get();
        // dd($conversations);

        return view('chat.index', [
            'roomId'        => $conversation->id,
            'messages'      => $messages,
            'conversation'  => $conversation,
            'conversations' => $conversations,
        ]);
    }

    public function store(Request $r, Conversation $conversation)
    {
        $this->authorize('send', $conversation);

        $data = $r->validate(['body' => 'required|string|max:5000']);

        $msg = $conversation->messages()->create([
            'user_id' => Auth::id(), // ✅
            'body'    => $data['body'],
        ])->load('user');

        broadcast(new MessageSent($msg))->toOthers();

        return response()->json(['message' => $msg]);
    }
}
