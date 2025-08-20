<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;
use Illuminate\Database\Eloquent\SoftDeletes;

class Load extends Models
{
    use HasFactory, SoftDeletes;
    protected $table = 'loads';
    protected $guarded = [];
    public $timestamps = true;

    public function user()
    {
        return $this->belongsTo(User::class, 'user_id');
    }

    public function load_type()
    {
        return $this->belongsTo(LoadType::class, 'load_type_id');
    }

    public function equipment()
    {
        return $this->belongsTo(Equipments::class, 'equipment_id');
    }

    public function commodity_type()
    {
        return $this->belongsTo(CommodityTypes::class, 'commodity_type_id');
    }

    public function legs()
    {
        return $this->hasMany(LoadLeg::class, 'load_id');
    }

}


