<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;
use Illuminate\Database\Eloquent\SoftDeletes;

class CommodityTypes extends Models
{
    use HasFactory, SoftDeletes;
    protected $table = 'commodity_types';
    protected $guarded = [];
    public $timestamps = true;

}

