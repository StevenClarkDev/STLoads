<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;
use Illuminate\Database\Eloquent\SoftDeletes;

class LoadLeg extends Models
{
    use HasFactory, SoftDeletes;
    protected $table = 'load_legs';
    protected $guarded = [];
    public $timestamps = true;

    protected $casts = [
        'pickup_date' => 'datetime',
        'delivery_date' => 'datetime',
    ];
    public function load_master()
    {
        return $this->belongsTo(Load::class, 'load_id');
    }

    public function status_master()
    {
        return $this->belongsTo(LoadLegStatusMaster::class, 'status_id');
    }
    public function carrier()
    {
        return $this->belongsTo(User::class, 'booked_carrier_id');
    }

    public function offer()
    {
        return $this->belongsTo(Offers::class, 'accepted_offer_id');
    }

    public function offers()
    {
        return $this->hasMany(Offers::class, 'load_leg_id');
    }

    public function pickupLocation()
    {
        return $this->belongsTo(Locations::class, 'pickup_location_id');
    }

    public function deliveryLocation()
    {
        return $this->belongsTo(Locations::class, 'delivery_location_id');
    }

    public function escrow()
    {
        return $this->hasOne(Escrow::class, 'leg_id', 'id');
    }

    public function locations()
    {
        return $this->hasMany(LoadLegLocation::class, 'leg_id', 'id');
    }
    public function events()
    {
        return $this->hasMany(LegEvent::class, 'leg_id', 'id');
    }
    public function documents()
    {
        return $this->hasMany(LegDocuments::class, 'leg_id', 'id');
    }


}
