<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class Country extends Models
{
    protected $table = 'countries';
    protected $guarded = [];
    public $timestamps = true;

}

