<?php

namespace App\Models;

// use Illuminate\Contracts\Auth\MustVerifyEmail;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Spatie\Permission\Traits\HasRoles;
use Laravel\Sanctum\HasApiTokens;

class User extends Authenticatable
{
    /** @use HasFactory<\Database\Factories\UserFactory> */
    use HasFactory, Notifiable, HasRoles, HasApiTokens;

    /**
     * The attributes that are mass assignable.
     *
     * @var list<string>
     */
    protected $fillable = [
        'name',
        'email',
        'password',
        'role_id',
        'dob',
        'gender',
        'phone_no',
        'ucr_hcc_no',
        'mc_cbsa_usdot_no',
        'ssn_no',
        'address',
        // shipper registration fields
        'nationality',
        'company_name',
        'registration_number',
        'tax_id',
        'country_of_incorporation',
        'consent_sanctions_screening',
        'politically_exposed_person',
        'source_of_funds',
        'agree_aml_policies',
        // carrier registration fields
        'gov_id_number',
        'cdl_number',
        'cdl_expiry',
        'cdl_class',
        'regulatory_country',
        'usdot_number',
        'mc_number',
        'ntn',
        'vat_number',
        'insurance_expiry',
        'coverage_limits',
        'insurer_name',
        'vehicle_reg',
        'vehicle_make_model',
        'vehicle_year',
        'vehicle_type',
        'load_capacity',
        'company_address',
        'bank_account',
        'criminal_declaration',
        'terms_agreed',
        // freight forwarder registration fields
        'trade_name',
        'incorporation_date',
        'director_name',
        'director_dob',
        'ubo_name',
        'ubo_dob',
        'ubo_nationality',
        'ubo_address',
        'fmc_license',
        'nvocc_reg',
        'surety_bond',
        'customs_broker_license',
        'iata_accreditation',
        'eori_number',
        'secp_reg',
        'chamber_reg',
        'policy_number',
        'insurer_contact',
        'transport_modes',
        'countries_served',
        'customs_brokerage',
        'consolidation_services',
        'warehousing',
        'years_in_operation',
        'annual_volume',
        'monthly_transaction_volume',
        'ofac_consent',
        // common
        'otp',
        'otp_expires_at',
        'otp_resend_count',
        'last_otp_resend_at',
        'image',
        'email_verified_at',
        'status',
        'approved_at',
        'rejected_at',
        'kyc_pending_at',
        'stripe_connect_account_id',
        'payouts_enabled',
        'kyc_status',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var list<string>
     */
    protected $hidden = [
        'password',
        'remember_token',
    ];

    /**
     * Get the attributes that should be cast.
     *
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'email_verified_at' => 'datetime',
            'password' => 'hashed',
        ];
    }

    public function carrierPreference()
    {
        return $this->hasOne(CarrierPreference::class, 'user_id');
    }
    public function kycDocuments()
    {
        return $this->hasMany(KycDocuments::class, 'user_id');
    }
    public function history()
    {
        return $this->hasMany(UserHistory::class, 'user_id');
    }
    public function latestHistory()
    {
        return $this->hasOne(UserHistory::class, 'user_id')->latestOfMany(); // uses created_at/id
    }
}