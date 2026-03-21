<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model as Models;

class LoadLegLocation extends Models
{
    protected $table = 'leg_locations';
    protected $guarded = [];
    public $timestamps = true;
    protected $fillable = [
        'leg_id',
        'lat',
        'lng',
        'recorded_at',
    ];

    protected $casts = [
        'recorded_at' => 'datetime',
        'lat' => 'float',
        'lng' => 'float',
    ];

    public function leg()
    {
        return $this->belongsTo(LoadLeg::class, 'leg_id');
    }


}
