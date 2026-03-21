<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model as Models;

class LegEvent extends Models
{
    protected $table = 'leg_events';
    protected $guarded = [];
    public $timestamps = true;

    protected $casts = [
        'created_at' => 'datetime',
    ];


    public function load_leg()
    {
        return $this->belongsTo(LoadLeg::class, 'leg_id');
    }


}
