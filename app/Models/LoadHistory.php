<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class LoadHistory extends Models
{
    protected $table = 'load_history';
    protected $guarded = [];
    public $timestamps = true;

}
