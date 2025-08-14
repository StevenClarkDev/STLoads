<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;
use Illuminate\Database\Eloquent\SoftDeletes;

class LoadType extends Models
{
    use HasFactory, SoftDeletes;
    protected $table = 'load_types';
    protected $guarded = [];
    public $timestamps = true;

}

