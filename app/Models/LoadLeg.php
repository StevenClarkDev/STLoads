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

    public function load_master()
    {
        return $this->belongsTo(Load::class, 'load_id');
    }

    public function pickupLocation()
    {
        return $this->belongsTo(Locations::class, 'pickup_location_id');
    }

    public function deliveryLocation()
    {
        return $this->belongsTo(Locations::class, 'delivery_location_id');
    }
}


