<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;
use Illuminate\Database\Eloquent\SoftDeletes;

class Locations extends Models
{
    use HasFactory, SoftDeletes;
    protected $table = 'locations';
    protected $guarded = [];
    public $timestamps = true;

}

