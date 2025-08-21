<?php
namespace App\Policies;

use App\Models\{Conversation, Load, User};
use App\Support\Roles;

class ConversationPolicy
{
    public function view(User $u, Conversation $c): bool {
        return $u->hasRole(Roles::ADMIN) || in_array($u->id, [$c->carrier_id,$c->shipper_id], true);
    }
    public function send(User $u, Conversation $c): bool { return $this->view($u, $c); }

    public function createFromLoad(User $u, Load $load): bool {
        if (! $u->hasRole(Roles::CARRIER)) return false;
        if (strtolower((string)$load->bid_status) === 'fixed') return false;
        $owner = $load->user; // adjust if relation name differs
        return $owner && $owner->hasAnyRole(Roles::LOAD_OWNERS);
    }
}
