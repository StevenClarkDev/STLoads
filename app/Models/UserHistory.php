<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class UserHistory extends Models
{
    protected $table = 'user_history';
    protected $guarded = [];
    public $timestamps = true;

}
