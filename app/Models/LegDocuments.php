<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model as Models;

class LegDocuments extends Models
{
    protected $table = 'leg_documents';
    protected $guarded = [];
    public $timestamps = true;

    protected $casts = [
        'meta' => 'array',
        'created_at' => 'datetime',
    ];


    public function load_leg()
    {
        return $this->belongsTo(LoadLeg::class, 'leg_id');
    }


}
