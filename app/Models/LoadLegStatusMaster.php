<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class LoadLegStatusMaster extends Models
{
    use HasFactory;
    protected $table = 'load_status_master';
    protected $guarded = [];

}

