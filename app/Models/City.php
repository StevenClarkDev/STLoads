<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class City extends Models
{
    protected $table = 'cities';
    protected $guarded = [];
    public $timestamps = true;

}
