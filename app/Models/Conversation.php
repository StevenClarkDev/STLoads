<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class Conversation extends Models
{
    protected $table = 'conversations';
    protected $guarded = [];
    public $timestamps = true;

    protected $fillable = ['load_id', 'shipper_id', 'carrier_id'];

    public function messages()
    {
        return $this->hasMany(Message::class);
    }
    public function shipper()
    {
        return $this->belongsTo(User::class, 'shipper_id');
    }
    public function carrier()
    {
        return $this->belongsTo(User::class, 'carrier_id');
    }
}
