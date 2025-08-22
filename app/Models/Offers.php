<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class Offers extends Models
{
    use HasFactory;
    protected $table = 'offers';
    protected $guarded = [];
    public $timestamps = true;

    public function carrier()
    {
        return $this->belongsTo(User::class, 'carrier_id');
    }

    public function status_master()
    {
        return $this->belongsTo(OfferStatusMaster::class, 'status_id');
    }

    public function loadLeg() {
        return $this->belongsTo(LoadLeg::class, 'load_leg_id');
    }
}


