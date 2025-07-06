<?php

namespace App\Models;

// use Illuminate\Contracts\Auth\MustVerifyEmail;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;

class Logs extends Authenticatable
{
    protected $connection = 'second_db';
    protected $table = 'logs';
    protected $guarded = [];
    public $timestamps = true;
}
