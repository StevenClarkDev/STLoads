<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class StloadsExternalRef extends Model
{
    protected $table = 'stloads_external_refs';

    protected $guarded = [];

    public function handoff()
    {
        return $this->belongsTo(StloadsHandoff::class, 'handoff_id');
    }
}
