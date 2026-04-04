<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class StloadsHandoffEvent extends Model
{
    protected $table = 'stloads_handoff_events';

    protected $guarded = [];

    public function handoff()
    {
        return $this->belongsTo(StloadsHandoff::class, 'handoff_id');
    }
}
