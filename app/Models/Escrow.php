<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class Escrow extends Models
{
    use HasFactory;

    protected $fillable = [
        'leg_id',
        'payer_user_id',
        'payee_user_id',
        'currency',
        'amount',
        'platform_fee',
        'status',
        'transfer_group',
        'payment_intent_id',
        'charge_id',
        'transfer_id',
    ];

    public function leg()
    {
        return $this->belongsTo(LoadLeg::class, 'leg_id');
    }
}
