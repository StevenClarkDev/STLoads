<?php
namespace App\Events;

use App\Models\Message;
use Illuminate\Broadcasting\PrivateChannel;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Queue\SerializesModels;

class MessageSent implements ShouldBroadcastNow
{
    use SerializesModels;

    public function __construct(public Message $message) {
        $this->message->load('user','conversation');
    }

    public function broadcastOn(): array {
        return [ new PrivateChannel('convo.'.$this->message->conversation_id) ];
    }
    public function broadcastAs(): string { return 'message.sent'; }
    public function broadcastWith(): array {
        return [
            'id'=>$this->message->id,
            'conversation_id'=>$this->message->conversation_id,
            'body'=>$this->message->body,
            'meta'=>$this->message->meta,
            'user_id'=>$this->message->user_id,
            'user'=>['id'=>$this->message->user->id, 'name'=>$this->message->user->name],
            'created_at'=>$this->message->created_at->toIso8601String(),
        ];
    }
}
