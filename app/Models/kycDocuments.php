<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model as Models;

class KycDocuments extends Models
{
    protected $table = 'kyc_documents';
    protected $guarded = [];
    public $timestamps = true;

    protected $fillable = ['user_id', 'document_type', 'file_path'];

    public function user()
    {
        return $this->belongsTo(User::class);
    }
}

