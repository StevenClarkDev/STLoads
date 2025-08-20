<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\SoftDeletes;


class ShipperDetail extends Model
{
    use HasFactory, SoftDeletes;
    protected $table = 'shipper_detail';
    protected $guarded = [];
    public $timestamps = true;

    public function user()
    {
        return $this->belongsTo(User::class);
    }
}
