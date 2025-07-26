<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class UserDetail extends Model
{
    use HasFactory;

    protected $fillable = [
        'user_id',
        'company_name',
        'company_address',
        'dot_number',
        'mc_number',
        'equipment_types',
        'business_entity_id',
        'facility_address',
        'fulfillment_contact_info',
        'fmcsa_broker_license_no',
        'mc_authority_number',
        'freight_forwarder_license',
        'customs_license',
    ];

    public function user()
    {
        return $this->belongsTo(User::class);
    }
}
